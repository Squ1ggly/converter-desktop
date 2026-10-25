use std::collections::VecDeque;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, ChildStderr, ChildStdout, Command};
use tokio_util::sync::CancellationToken;

use crate::child_lifetime;
use crate::error::{ConversionError, ConversionResult};

// Enough of FFmpeg's log to find the real error, which is usually near the end.
const STDERR_LINES_KEPT: usize = 40;
const MAX_REPORTED_PERCENT: f32 = 99.0;

// Matches externalBin in tauri.conf.json. Not plain "ffmpeg", so Linux packages
// don't install a /usr/bin/ffmpeg that clashes with the distro's FFmpeg.
const SIDECAR_NAME: &str = "converter-ffmpeg";

// The bundled FFmpeg. Tauri copies sidecars next to the app executable.
#[derive(Clone, Debug)]
pub struct Ffmpeg {
    binary_path: PathBuf,
}

impl Ffmpeg {
    pub fn locate() -> ConversionResult<Ffmpeg> {
        let current_executable = std::env::current_exe()?;
        let executable_directory = current_executable
            .parent()
            .unwrap_or(current_executable.as_path());
        let binary_name = format!("{SIDECAR_NAME}{}", std::env::consts::EXE_SUFFIX);
        let binary_path = executable_directory.join(binary_name);

        if !binary_path.is_file() {
            return Err(ConversionError::MissingFfmpeg(binary_path));
        }

        let ffmpeg = Ffmpeg { binary_path };
        Ok(ffmpeg)
    }

    #[cfg(test)]
    pub fn binary_path(&self) -> &Path {
        &self.binary_path
    }

    // Tests run from target/debug/deps, where no sidecar is copied.
    #[cfg(test)]
    pub fn from_downloaded_binary() -> Ffmpeg {
        let binaries_directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("binaries");
        let sidecar_prefix = format!("{SIDECAR_NAME}-");
        let missing_message = "No FFmpeg in src-tauri/binaries. Run `pnpm fetch-ffmpeg` first.";

        let binary_path = std::fs::read_dir(&binaries_directory)
            .expect(missing_message)
            .filter_map(|entry| entry.ok().map(|entry| entry.path()))
            .find(|path| file_name_starts_with(path, &sidecar_prefix))
            .expect(missing_message);

        Ffmpeg { binary_path }
    }

    // Reports 0–99% progress when the input has a known duration.
    // Cancelling kills the process and waits for it to exit.
    pub async fn run(
        &self,
        arguments: Vec<OsString>,
        cancellation: &CancellationToken,
        on_progress: impl FnMut(f32),
    ) -> ConversionResult<()> {
        let path_arguments = absolute_path_arguments(&arguments);
        let mut command = Command::new(&self.binary_path);
        command
            .args([
                "-hide_banner",
                "-nostdin",
                "-y",
                "-nostats",
                "-progress",
                "pipe:1",
            ])
            .args(arguments)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);
        hide_console_window(&mut command);

        let mut child = command.spawn()?;
        child_lifetime::end_with_app(&child);
        let (Some(stdout), Some(stderr)) = (child.stdout.take(), child.stderr.take()) else {
            stop_process(&mut child).await;
            return Err(std::io::Error::other("FFmpeg's output couldn't be read").into());
        };

        let total_duration_microseconds = Arc::new(AtomicU64::new(0));
        let stderr_task = tokio::spawn(collect_stderr(stderr, total_duration_microseconds.clone()));

        let run_to_completion = async {
            report_progress(stdout, &total_duration_microseconds, on_progress).await;
            child.wait().await
        };
        let finished_exit_status = tokio::select! {
            exit_status = run_to_completion => Some(exit_status),
            () = cancellation.cancelled() => None,
        };

        let Some(exit_status) = finished_exit_status else {
            stop_process(&mut child).await;
            return Err(ConversionError::Cancelled);
        };
        let exit_status = exit_status?;

        if exit_status.success() {
            return Ok(());
        }

        let stderr_lines = stderr_task.await.unwrap_or_default();
        log::warn!("FFmpeg failed:\n{}", stderr_lines.join("\n"));
        let error_message = extract_error_message(&stderr_lines, &path_arguments);
        Err(ConversionError::Ffmpeg(error_message))
    }
}

// Waits for the process to end, so its files are closed before the caller
// deletes them. Windows can't delete a file another process has open.
async fn stop_process(child: &mut Child) {
    if let Err(error) = child.kill().await {
        log::warn!("couldn't stop FFmpeg: {error}");
    }
}

#[cfg(test)]
fn file_name_starts_with(path: &Path, prefix: &str) -> bool {
    let file_name = path
        .file_name()
        .map(|name| name.to_string_lossy())
        .unwrap_or_default();
    file_name.starts_with(prefix)
}

#[cfg(windows)]
fn hide_console_window(command: &mut Command) {
    // Without this, every conversion flashes a console window on Windows.
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    command.creation_flags(CREATE_NO_WINDOW);
}

#[cfg(not(windows))]
fn hide_console_window(_command: &mut Command) {}

async fn report_progress(
    stdout: ChildStdout,
    total_duration_microseconds: &AtomicU64,
    mut on_progress: impl FnMut(f32),
) {
    let mut lines = BufReader::new(stdout).lines();

    while let Ok(Some(line)) = lines.next_line().await {
        let total_microseconds = total_duration_microseconds.load(Ordering::Relaxed);
        let Some(percent) = parse_progress_percent(&line, total_microseconds) else {
            continue;
        };
        on_progress(percent);
    }
}

// Drains stderr so FFmpeg can't block on a full pipe. Keeps the duration (for
// progress) and the last lines (for errors).
async fn collect_stderr(
    stderr: ChildStderr,
    total_duration_microseconds: Arc<AtomicU64>,
) -> Vec<String> {
    let mut recent_lines = VecDeque::with_capacity(STDERR_LINES_KEPT);
    let mut lines = BufReader::new(stderr).lines();

    while let Ok(Some(line)) = lines.next_line().await {
        record_duration_if_present(&line, &total_duration_microseconds);

        if recent_lines.len() == STDERR_LINES_KEPT {
            recent_lines.pop_front();
        }
        recent_lines.push_back(line);
    }

    let kept_lines = Vec::from(recent_lines);
    kept_lines
}

fn record_duration_if_present(line: &str, total_duration_microseconds: &AtomicU64) {
    let is_duration_known = total_duration_microseconds.load(Ordering::Relaxed) > 0;
    if is_duration_known {
        return;
    }

    if let Some(duration_microseconds) = parse_duration_microseconds(line) {
        total_duration_microseconds.store(duration_microseconds, Ordering::Relaxed);
    }
}

// Parses FFmpeg's input summary, e.g. "  Duration: 00:01:23.45, start: ...".
fn parse_duration_microseconds(line: &str) -> Option<u64> {
    let after_label = line.trim_start().strip_prefix("Duration: ")?;
    let timestamp = after_label.split(',').next()?;
    let mut parts = timestamp.split(':');

    let hours: f64 = parts.next()?.parse().ok()?;
    let minutes: f64 = parts.next()?.parse().ok()?;
    let seconds: f64 = parts.next()?.parse().ok()?;

    let total_seconds = hours * 3600.0 + minutes * 60.0 + seconds;
    if total_seconds <= 0.0 {
        return None;
    }

    let duration_microseconds = (total_seconds * 1_000_000.0).round() as u64;
    Some(duration_microseconds)
}

// Parses one `-progress` line, e.g. "out_time_us=1234567".
fn parse_progress_percent(line: &str, total_microseconds: u64) -> Option<f32> {
    if total_microseconds == 0 {
        return None;
    }

    let elapsed_microseconds: u64 = line.trim().strip_prefix("out_time_us=")?.parse().ok()?;
    let fraction_done = elapsed_microseconds as f64 / total_microseconds as f64;
    let percent = (fraction_done * 100.0) as f32;

    let clamped_percent = percent.clamp(0.0, MAX_REPORTED_PERCENT);
    Some(clamped_percent)
}

// FFmpeg's own bookkeeping, repeated after every real error.
const NOISE_LINE_MARKERS: [&str; 2] = [
    "Terminating thread with return code",
    "Task finished with error code",
];

fn extract_error_message(stderr_lines: &[String], path_arguments: &[String]) -> String {
    let cleaned_lines: Vec<String> = stderr_lines
        .iter()
        .filter(|line| {
            !NOISE_LINE_MARKERS
                .iter()
                .any(|marker| line.contains(marker))
        })
        .map(|line| clean_log_line(line, path_arguments))
        .filter(|line| !line.is_empty())
        .collect();

    let explicit_error_line = cleaned_lines.iter().rev().find(|line| {
        line.contains("Error")
            || line.contains("Invalid")
            || line.contains("No such")
            || line.contains("not supported")
    });
    if let Some(error_line) = explicit_error_line {
        return error_line.clone();
    }

    let last_lines: Vec<&str> = cleaned_lines
        .iter()
        .rev()
        .take(2)
        .rev()
        .map(String::as_str)
        .collect();
    let fallback_message = if last_lines.is_empty() {
        "FFmpeg could not convert this file".to_string()
    } else {
        last_lines.join(" ")
    };
    fallback_message
}

fn absolute_path_arguments(arguments: &[OsString]) -> Vec<String> {
    let path_arguments = arguments
        .iter()
        .filter(|argument| Path::new(argument).is_absolute())
        .filter_map(|argument| argument.to_str().map(str::to_string))
        .collect();
    path_arguments
}

// Drops "[out#0/image2 @ 0x6000038c4000] " style prefixes, and shortens the
// full paths FFmpeg was given to their file names.
fn clean_log_line(line: &str, path_arguments: &[String]) -> String {
    let mut message = line.trim();
    while message.starts_with('[') {
        let Some(prefix_end) = message.find("] ") else {
            break;
        };
        message = message[prefix_end + 2..].trim_start();
    }

    let mut cleaned_line = message.to_string();
    for path_argument in path_arguments {
        let file_name = Path::new(path_argument)
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_default();
        cleaned_line = cleaned_line.replace(path_argument.as_str(), &file_name);
    }
    cleaned_line
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_input_duration() {
        let line = "  Duration: 00:01:30.50, start: 0.000000, bitrate: 1205 kb/s";
        assert_eq!(parse_duration_microseconds(line), Some(90_500_000));
        assert_eq!(
            parse_duration_microseconds("  Duration: N/A, bitrate: N/A"),
            None
        );
        assert_eq!(
            parse_duration_microseconds("Stream #0:0: Video: h264"),
            None
        );
    }

    #[test]
    fn progress_is_capped_below_one_hundred_until_ffmpeg_exits() {
        assert_eq!(
            parse_progress_percent("out_time_us=5000000", 10_000_000),
            Some(50.0)
        );
        assert_eq!(
            parse_progress_percent("out_time_us=20000000", 10_000_000),
            Some(MAX_REPORTED_PERCENT)
        );
        assert_eq!(parse_progress_percent("out_time_us=5000000", 0), None);
        assert_eq!(parse_progress_percent("frame=12", 10_000_000), None);
    }

    #[test]
    fn prefers_the_last_explicit_error_line() {
        let lines = vec![
            "Input #0, mov".to_string(),
            "Error opening input file broken.mov.".to_string(),
            "Exiting".to_string(),
        ];
        assert_eq!(
            extract_error_message(&lines, &[]),
            "Error opening input file broken.mov."
        );
    }

    #[test]
    fn error_lines_lose_log_prefixes_addresses_and_folders() {
        let output_path = "/Users/sam/Pictures/Week 3/photo.webp".to_string();
        let lines = vec![
            "[out#0/image2 @ 0x6000038c4000] [enc:png @ 0x1319] Error opening output /Users/sam/Pictures/Week 3/photo.webp: Permission denied".to_string(),
            "[vost#0:0/png @ 0x1319051e0] Task finished with error code: -22 (Invalid argument)".to_string(),
            "[vf#0:0 @ 0x13200a600] Terminating thread with return code -22 (Invalid argument)".to_string(),
        ];

        let message = extract_error_message(&lines, &[output_path]);

        assert_eq!(
            message,
            "Error opening output photo.webp: Permission denied"
        );
    }
}
