// Child processes must not outlive the app. Quitting normally cancels every
// conversion first; this also covers a crash or a forced quit on Windows,
// where the system closes the job when the app ends and kills what's in it.

use tokio::process::Child;

#[cfg(windows)]
pub fn end_with_app(child: &Child) {
    let Some(process_handle) = child.raw_handle() else {
        return;
    };
    let Some(job) = windows_job::kill_on_close_job() else {
        return;
    };
    // SAFETY: both handles are valid; the job lives as long as the app and the
    // process handle as long as `child`.
    let is_assigned = unsafe {
        windows_sys::Win32::System::JobObjects::AssignProcessToJobObject(
            job.handle(),
            process_handle,
        )
    } != 0;
    if !is_assigned {
        log::warn!("couldn't tie a child process to the app's lifetime");
    }
}

#[cfg(not(windows))]
pub fn end_with_app(_child: &Child) {}

#[cfg(windows)]
pub mod windows_job {
    use std::sync::OnceLock;

    use windows_sys::Win32::Foundation::HANDLE;
    use windows_sys::Win32::System::JobObjects::{
        CreateJobObjectW, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE, JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
        JobObjectExtendedLimitInformation, SetInformationJobObject,
    };

    pub struct JobHandle(HANDLE);

    // SAFETY: a job handle can be used from any thread.
    unsafe impl Send for JobHandle {}
    unsafe impl Sync for JobHandle {}

    impl JobHandle {
        pub fn handle(&self) -> HANDLE {
            self.0
        }
    }

    // Created once and never closed: the system closes it when the app ends.
    pub fn kill_on_close_job() -> Option<&'static JobHandle> {
        static JOB: OnceLock<Option<JobHandle>> = OnceLock::new();
        JOB.get_or_init(create_kill_on_close_job).as_ref()
    }

    fn create_kill_on_close_job() -> Option<JobHandle> {
        // SAFETY: null arguments create an unnamed job with default security.
        let job = unsafe { CreateJobObjectW(std::ptr::null(), std::ptr::null()) };
        if job.is_null() {
            log::warn!("couldn't create a job object for child processes");
            return None;
        }

        let mut limits = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
        limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
        let limits_size = u32::try_from(size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>()).ok()?;
        // SAFETY: `limits` is a valid JOBOBJECT_EXTENDED_LIMIT_INFORMATION of `limits_size` bytes.
        let is_configured = unsafe {
            SetInformationJobObject(
                job,
                JobObjectExtendedLimitInformation,
                (&raw const limits).cast(),
                limits_size,
            )
        } != 0;
        if !is_configured {
            log::warn!("couldn't set the job object to kill child processes");
            return None;
        }

        Some(JobHandle(job))
    }
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;
    use windows_sys::Win32::System::JobObjects::IsProcessInJob;

    #[tokio::test]
    async fn child_processes_are_placed_in_the_kill_on_close_job() {
        let mut child = tokio::process::Command::new("cmd")
            .args(["/C", "ping -n 30 127.0.0.1 > NUL"])
            .kill_on_drop(true)
            .spawn()
            .expect("start a child process");

        end_with_app(&child);

        let job = windows_job::kill_on_close_job().expect("job exists");
        let mut is_in_job = 0;
        // SAFETY: both handles are valid for the duration of the call.
        unsafe { IsProcessInJob(child.raw_handle().unwrap(), job.handle(), &mut is_in_job) };
        let _ = child.kill().await;
        assert_ne!(is_in_job, 0);
    }
}
