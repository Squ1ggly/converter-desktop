use std::path::{Path, PathBuf};

use serde::Serialize;
use tauri::ipc::Channel;
use tauri::{AppHandle, Manager, State};
use tauri_plugin_opener::OpenerExt;

use crate::batch::{BatchContext, ConversionEvent, ConversionRequest, run_batch};
use crate::formats::{ConversionCatalog, MediaKind, build_catalog, classify_path};
use crate::state::AppState;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InspectedFile {
    path: PathBuf,
    name: String,
    size_bytes: u64,
    // None for files the app can't convert.
    kind: Option<MediaKind>,
}

#[tauri::command]
pub fn get_catalog() -> ConversionCatalog {
    build_catalog()
}

// Folders are searched recursively and their unsupported files skipped.
// Loose files are always returned so the UI can report unsupported ones.
#[tauri::command]
pub fn inspect_paths(app: AppHandle, paths: Vec<PathBuf>) -> Vec<InspectedFile> {
    let inspected_files: Vec<InspectedFile> = expand_folders(paths)
        .into_iter()
        .filter_map(|path| inspect_file(&path))
        .collect();

    // Allows image previews through the asset protocol, one file at a time.
    let asset_scope = app.asset_protocol_scope();
    for file in inspected_files
        .iter()
        .filter(|file| file.kind == Some(MediaKind::Image))
    {
        let _ = asset_scope.allow_file(&file.path);
    }

    inspected_files
}

fn expand_folders(paths: Vec<PathBuf>) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let mut pending_folders = Vec::new();

    for path in paths {
        if path.is_dir() {
            pending_folders.push(path);
        } else {
            files.push(path);
        }
    }

    while let Some(folder) = pending_folders.pop() {
        for entry_path in list_visible_entries(&folder) {
            if entry_path.is_dir() {
                pending_folders.push(entry_path);
            } else if classify_path(&entry_path).is_some() {
                files.push(entry_path);
            }
        }
    }

    files
}

fn list_visible_entries(folder: &Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(folder) else {
        return Vec::new();
    };

    let mut entry_paths: Vec<PathBuf> = entries
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| !is_hidden(path))
        .collect();
    entry_paths.sort();
    entry_paths
}

fn is_hidden(path: &Path) -> bool {
    let is_dot_file = path
        .file_name()
        .map(|name| name.to_string_lossy().starts_with('.'))
        .unwrap_or(false);
    is_dot_file
}

fn inspect_file(path: &Path) -> Option<InspectedFile> {
    let metadata = std::fs::metadata(path).ok()?;
    let name = path.file_name()?.to_string_lossy().into_owned();

    let inspected_file = InspectedFile {
        path: path.to_path_buf(),
        name,
        size_bytes: metadata.len(),
        kind: classify_path(path),
    };
    Some(inspected_file)
}

#[tauri::command]
pub async fn convert_files(
    state: State<'_, AppState>,
    request: ConversionRequest,
    on_event: Channel<ConversionEvent>,
) -> Result<(), String> {
    let ffmpeg = state.ffmpeg.clone()?;
    let batch_id = request.batch_id.clone();

    let context = BatchContext {
        ffmpeg,
        scratch_root: state.scratch_root.clone(),
        cancellation: state.active_batches.start(&batch_id),
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
