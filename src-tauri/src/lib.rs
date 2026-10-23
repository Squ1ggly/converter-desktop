mod batch;
mod commands;
#[cfg(test)]
mod conversion_tests;
mod error;
mod ffmpeg;
mod formats;
mod image_conversion;
mod options;
mod output_path;
mod state;
mod video_conversion;

use std::path::PathBuf;
use std::sync::Arc;

use tauri::{App, Manager, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_log::{Target, TargetKind};

use crate::ffmpeg::Ffmpeg;
use crate::state::{ActiveBatches, AppState, ProducedOutputs};

// Must match the window listed in capabilities/default.json.
const MAIN_WINDOW_LABEL: &str = "main";

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(
            // Logs go to the per-user log folder (e.g. ~/Library/Logs/<app id>
            // on macOS, %LOCALAPPDATA%\<app id>\logs on Windows).
            tauri_plugin_log::Builder::new()
                .targets([Target::new(TargetKind::LogDir { file_name: None }), Target::new(TargetKind::Stdout)])
                .level(log::LevelFilter::Info)
                .build(),
        )
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .setup(|app| {
            let app_state = create_app_state(app)?;
            app.manage(app_state);
            create_main_window(app)?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_catalog,
            commands::inspect_paths,
            commands::convert_files,
            commands::cancel_conversion,
            commands::reveal_output,
            commands::open_output,
        ])
        .run(tauri::generate_context!())
        .expect("error while running the Converter app");
}

// Built here rather than in tauri.conf.json so the macOS-only title bar style
// doesn't need a second, duplicated window definition per platform.
fn create_main_window(app: &App) -> tauri::Result<()> {
    let window_builder = WebviewWindowBuilder::new(app, MAIN_WINDOW_LABEL, WebviewUrl::default())
        .title("Converter")
        .inner_size(1080.0, 720.0)
        .min_inner_size(760.0, 540.0);

    // Lets the app draw its own toolbar under the traffic-light buttons, as
    // native macOS apps do.
    #[cfg(target_os = "macos")]
    let window_builder = window_builder
        .title_bar_style(tauri::TitleBarStyle::Overlay)
        .hidden_title(true);

    window_builder.build()?;
    Ok(())
}

fn create_app_state(app: &App) -> Result<AppState, Box<dyn std::error::Error>> {
    let ffmpeg = Ffmpeg::locate().map_err(|error| error.to_string());
    if let Err(message) = &ffmpeg {
        log::error!("{message}");
    }

    let app_state = AppState {
        ffmpeg,
        scratch_root: prepare_scratch_root(app)?,
        active_batches: ActiveBatches::default(),
        produced_outputs: Arc::new(ProducedOutputs::default()),
    };
    Ok(app_state)
}

// Intermediate files live in the current user's cache folder, not the
// system-wide temp folder. Anything left by a crash is cleared on launch.
fn prepare_scratch_root(app: &App) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let scratch_root = app.path().app_cache_dir()?.join("scratch");

    if scratch_root.exists() {
        std::fs::remove_dir_all(&scratch_root)?;
    }
    std::fs::create_dir_all(&scratch_root)?;

    Ok(scratch_root)
}
