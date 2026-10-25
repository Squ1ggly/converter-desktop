use std::path::{Path, PathBuf};

use tauri::ipc::Channel;
use tauri::{AppHandle, Manager, State};
use tauri_plugin_opener::OpenerExt;

use crate::batch::{BatchContext, ConversionEvent, ConversionRequest, run_batch};
use crate::formats::{ConversionCatalog, MediaKind, build_catalog};
use crate::input_files::{self, InspectedFile, PathInspection};
use crate::state::AppState;

#[tauri::command]
pub fn get_catalog() -> ConversionCatalog {
    build_catalog()
}

// Runs off the main thread: a large folder takes a while to read, and the
// window would stop responding meanwhile.
#[tauri::command]
pub async fn inspect_input_paths(
    app: AppHandle,
    paths: Vec<PathBuf>,
) -> Result<PathInspection, String> {
    let inspection =
        tauri::async_runtime::spawn_blocking(move || input_files::inspect_paths(paths))
            .await
            .map_err(|error| error.to_string())?;
    allow_image_previews(&app, &inspection.files);
    Ok(inspection)
}

// The original command, returning only the files. Kept until the UI calls
// inspect_input_paths, which also says when the file limit was reached.
#[tauri::command]
pub async fn inspect_paths(
    app: AppHandle,
    paths: Vec<PathBuf>,
) -> Result<Vec<InspectedFile>, String> {
    let inspection = inspect_input_paths(app, paths).await?;
    Ok(inspection.files)
}

// The webview can load these files through the asset protocol, one file at a
// time, for thumbnails. Nothing else is readable.
fn allow_image_previews(app: &AppHandle, files: &[InspectedFile]) {
    let asset_scope = app.asset_protocol_scope();
    let image_files = files
        .iter()
        .filter(|file| file.kind == Some(MediaKind::Image));
    for file in image_files {
        if let Err(error) = asset_scope.allow_file(&file.path) {
            log::warn!(
                "couldn't allow a preview of {}: {error}",
                file.path.display()
            );
        }
    }
}

#[tauri::command]
pub async fn convert_files(
    state: State<'_, AppState>,
    request: ConversionRequest,
    on_event: Channel<ConversionEvent>,
) -> Result<(), String> {
    let ffmpeg = state.ffmpeg.clone()?;
    let batch_id = request.batch_id.clone();
    let cancellation = state.active_batches.start(&batch_id)?;

    let context = BatchContext {
        ffmpeg,
        scratch_root: state.scratch_root.clone(),
        cancellation,
        events: on_event,
        produced_outputs: state.produced_outputs.clone(),
    };
    run_batch(request, context).await;

    state.active_batches.finish(&batch_id);
    Ok(())
}

#[tauri::command]
pub fn cancel_conversion(state: State<'_, AppState>, batch_id: String) {
    state.active_batches.cancel(&batch_id);
}

#[tauri::command]
pub fn reveal_output(
    app: AppHandle,
    state: State<'_, AppState>,
    path: PathBuf,
) -> Result<(), String> {
    ensure_produced_by_app(&state, &path)?;
    app.opener()
        .reveal_item_in_dir(&path)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn open_output(
    app: AppHandle,
    state: State<'_, AppState>,
    path: PathBuf,
) -> Result<(), String> {
    ensure_produced_by_app(&state, &path)?;
    let path_text = path.to_string_lossy().into_owned();
    app.opener()
        .open_path(path_text, None::<&str>)
        .map_err(|error| error.to_string())
}

fn ensure_produced_by_app(state: &AppState, path: &Path) -> Result<(), String> {
    if state.produced_outputs.contains(path) {
        return Ok(());
    }
    Err("Only files converted in this session can be opened from here".to_string())
}
