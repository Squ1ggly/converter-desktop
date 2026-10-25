use std::path::{Path, PathBuf};
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tauri::ipc::Channel;
use tokio::sync::Semaphore;
use tokio_util::sync::CancellationToken;

use crate::error::{ConversionError, ConversionResult};
use crate::ffmpeg::Ffmpeg;
use crate::image_conversion::{ImageConversion, convert_image};
use crate::options::ConversionSettings;
use crate::output_path::reserve_output_path;
use crate::state::ProducedOutputs;
use crate::video_conversion::{VideoConversion, convert_video};

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestedFile {
    pub file_id: String,
    pub path: PathBuf,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversionRequest {
    pub batch_id: String,
    pub files: Vec<RequestedFile>,
    pub settings: ConversionSettings,
    // None saves each result next to its original.
    pub output_directory: Option<PathBuf>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum ConversionEvent {
    Started {
        file_id: String,
    },
    Progress {
        file_id: String,
        percent: f32,
    },
    Completed {
        file_id: String,
        output_path: PathBuf,
    },
    Failed {
        file_id: String,
        message: String,
    },
    Cancelled {
        file_id: String,
    },
}

#[derive(Clone)]
pub struct BatchContext {
    pub ffmpeg: Ffmpeg,
    pub scratch_root: PathBuf,
    pub cancellation: CancellationToken,
    pub events: Channel<ConversionEvent>,
    pub produced_outputs: Arc<ProducedOutputs>,
}

#[derive(Clone)]
struct FileJob {
    file: RequestedFile,
    settings: ConversionSettings,
    output_directory: Option<PathBuf>,
}

// Returns once every file has completed, failed or been cancelled.
pub async fn run_batch(request: ConversionRequest, context: BatchContext) {
    let parallel_limit = Arc::new(Semaphore::new(request.settings.max_parallel_files()));

    let file_tasks: Vec<_> = request
        .files
        .into_iter()
        .map(|file| {
            let job = FileJob {
                file,
                settings: request.settings,
                output_directory: request.output_directory.clone(),
            };
            tauri::async_runtime::spawn(convert_queued_file(
                job,
                parallel_limit.clone(),
                context.clone(),
            ))
        })
        .collect();

    for file_task in file_tasks {
        let _ = file_task.await;
    }
}

async fn convert_queued_file(job: FileJob, parallel_limit: Arc<Semaphore>, context: BatchContext) {
    let file_id = job.file.file_id.clone();

    let permit = tokio::select! {
        permit = parallel_limit.acquire_owned() => permit.ok(),
        _ = context.cancellation.cancelled() => None,
    };
    let Some(_permit) = permit else {
        send_event(&context, ConversionEvent::Cancelled { file_id });
        return;
    };

    send_event(
        &context,
        ConversionEvent::Started {
            file_id: file_id.clone(),
        },
    );
    let conversion_result = convert_one_file(&job, &context).await;

    let finished_event = match conversion_result {
        Ok(output_path) => {
            context.produced_outputs.remember(&output_path);
            ConversionEvent::Completed {
                file_id,
                output_path,
            }
        }
        Err(ConversionError::Cancelled) => ConversionEvent::Cancelled { file_id },
        Err(error) => {
            log::warn!("conversion failed for {}: {error}", job.file.path.display());
            ConversionEvent::Failed {
                file_id,
                message: error.to_string(),
            }
        }
    };
    send_event(&context, finished_event);
}

// Removes the partial output if the conversion fails or is cancelled.
async fn convert_one_file(job: &FileJob, context: &BatchContext) -> ConversionResult<PathBuf> {
    let input_path =
        resolve_absolute_path(&job.file.path).map_err(|error| error.about_input(&job.file.path))?;
    let output_directory = resolve_output_directory(&input_path, job.output_directory.as_deref())?;
    let output_path = reserve_output_path(
        &input_path,
        &output_directory,
        job.settings.output_extension(),
    )
    .map_err(|error| error.about_output_folder(&output_directory))?;

    let conversion_result = run_conversion(job, &input_path, &output_path, context)
        .await
        .map_err(|error| error.about_conversion(&input_path, &output_directory))
        .and_then(|()| ensure_output_was_written(&output_path));
    if let Err(error) = conversion_result {
        let _ = std::fs::remove_file(&output_path);
        return Err(error);
    }

    Ok(output_path)
}

// The name is reserved by creating an empty file, so an encoder that wrote
// somewhere else would otherwise look like a success.
fn ensure_output_was_written(output_path: &Path) -> ConversionResult<()> {
    let output_size_bytes = std::fs::metadata(output_path)?.len();
    if output_size_bytes == 0 {
        return Err(ConversionError::EmptyOutput);
    }
    Ok(())
}

// The UI only sends absolute paths, from the file dialog or drag and drop.
// Resolving them means links and ".." can't point the conversion elsewhere.
fn resolve_absolute_path(path: &Path) -> ConversionResult<PathBuf> {
    if !path.is_absolute() {
        return Err(ConversionError::RelativePath);
    }
    let resolved_path = dunce::canonicalize(path)?;
    Ok(resolved_path)
}

// `input_path` is already resolved.
fn resolve_output_directory(
    input_path: &Path,
    chosen_directory: Option<&Path>,
) -> ConversionResult<PathBuf> {
    if let Some(chosen_directory) = chosen_directory {
        let resolved_directory = resolve_absolute_path(chosen_directory)
            .map_err(|error| error.about_output_folder(chosen_directory))?;
        return Ok(resolved_directory);
    }

    let source_directory = input_path.parent().map(Path::to_path_buf).ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "The file's folder could not be found",
        )
    })?;
    Ok(source_directory)
}

async fn run_conversion(
    job: &FileJob,
    input_path: &Path,
    output_path: &Path,
    context: &BatchContext,
) -> ConversionResult<()> {
    match job.settings {
        ConversionSettings::Image { format, options } => {
            let conversion = ImageConversion {
                input_path,
                output_path,
                format,
                options,
                ffmpeg: &context.ffmpeg,
                scratch_root: &context.scratch_root,
                cancellation: &context.cancellation,
            };
            convert_image(conversion).await
        }
        ConversionSettings::Video { format, options } => {
            let conversion = VideoConversion {
                input_path,
                output_path,
                format,
                options,
                ffmpeg: &context.ffmpeg,
                scratch_root: &context.scratch_root,
                cancellation: &context.cancellation,
            };
            let report_progress = |percent| {
                let file_id = job.file.file_id.clone();
                send_event(context, ConversionEvent::Progress { file_id, percent });
            };
            convert_video(conversion, report_progress).await
        }
    }
}

// The window may have closed mid-batch; there's nobody left to tell.
fn send_event(context: &BatchContext, event: ConversionEvent) {
    let _ = context.events.send(event);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_empty_output_is_not_a_success() {
        let directory = tempfile::tempdir().unwrap();
        let output_path = directory.path().join("photo.webp");
        std::fs::write(&output_path, b"").unwrap();

        let check_result = ensure_output_was_written(&output_path);

        assert!(matches!(check_result, Err(ConversionError::EmptyOutput)));
    }
}
