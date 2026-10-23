use std::collections::VecDeque;
use std::ffi::OsString;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{ChildStderr, ChildStdout, Command};
use tokio_util::sync::CancellationToken;

use crate::error::{ConversionError, ConversionResult};

// Enough of FFmpeg's log to find the real error, which is usually near the end.
const STDERR_LINES_KEPT: usize = 40;
const MAX_REPORTED_PERCENT: f32 = 99.0;

// The bundled FFmpeg binary. Tauri copies `externalBin` sidecars next to the
// app's own executable (in both `tauri dev` and installed builds), with the
// target-triple suffix stripped.
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
        let binary_name = format!("ffmpeg{}", std::env::consts::EXE_SUFFIX);
        let binary_path = executable_directory.join(binary_name);

        if !binary_path.is_file() {
            return Err(ConversionError::MissingFfmpeg(binary_path));
        }

        let ffmpeg = Ffmpeg { binary_path };
        Ok(ffmpeg)
    }

    // Tests run from target/debug/deps, where no sidecar is copied, so they
    // point straight at the binary `bun run fetch-ffmpeg` downloaded.
    #[cfg(test)]
    pub fn from_downloaded_binary() -> Ffmpeg {
        let binaries_directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("binaries");
        let binary_path = std::fs::read_dir(&binaries_directory)
            .ok()
            .and_then(|entries| {
                entries
                    .filter_map(|entry| entry.ok().map(|entry| entry.path()))
                    .find(|path| {
                        path.file_name()
                            .is_some_and(|name| name.to_string_lossy().starts_with("ffmpeg-"))
                    })
            })
            .expect("No FFmpeg in src-tauri/binaries. Run `bun run fetch-ffmpeg` first.");

        Ffmpeg { binary_path }
    }

    // Runs FFmpeg with `arguments` (everything after the global flags: inputs,
    // codecs, output). Reports 0–99 percent while running when the input has a
    // known duration. Cancelling kills the process.
    pub async fn run(
        &self,
        arguments: Vec<OsString>,
        cancellation: &CancellationToken,
        on_progress: impl FnMut(f32),
    ) -> ConversionResult<()> {
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
        let stdout = child.stdout.take().expect("stdout is piped");
        let stderr = child.stderr.take().expect("stderr is piped");

        let total_duration_microseconds = Arc::new(AtomicU64::new(0));
        let stderr_task = tokio::spawn(collect_stderr(stderr, total_duration_microseconds.clone()));

        let run_to_completion = async {
            report_progress(stdout, &total_duration_microseconds, on_progress).await;
            child.wait().await
        };

        // Returning early drops `child`, and `kill_on_drop` stops the process.
        let exit_status = tokio::select! {
            exit_status = run_to_completion => exit_status?,
            _ = cancellation.cancelled() => return Err(ConversionError::Cancelled),
        };

        if exit_status.success() {
            return Ok(());
        }

        let stderr_lines = stderr_task.await.unwrap_or_default();
        let error_message = extract_error_message(&stderr_lines);
        Err(ConversionError::Ffmpeg(error_message))
    }
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

// Drains stderr so FFmpeg never blocks on a full pipe, recording the input's
// duration (for progress) and the last lines of output (for error messages).
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

fn extract_error_message(stderr_lines: &[String]) -> String {
    let explicit_error_line = stderr_lines.iter().rev().find(|line| {
        line.contains("Error")
            || line.contains("Invalid")
            || line.contains("No such")
            || line.contains("not supported")
    });

    if let Some(error_line) = explicit_error_line {
        return error_line.trim().to_string();
    }

    let last_lines: Vec<&str> = stderr_lines
        .iter()
        .rev()
        .take(2)
        .rev()
        .map(|line| line.trim())
        .collect();
    let fallback_message = if last_lines.is_empty() {
        "FFmpeg could not convert this file".to_string()
    } else {
        last_lines.join(" ")
    };
    fallback_message
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
            extract_error_message(&lines),
            "Error opening input file broken.mov."
        );
    }
}
