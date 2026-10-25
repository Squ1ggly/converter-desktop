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
mod scratch;
mod state;
mod video_conversion;

use std::sync::Arc;

use tauri::{App, AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_log::{Target, TargetKind};

use crate::ffmpeg::Ffmpeg;
use crate::state::{ActiveBatches, AppState, ProducedOutputs};

// Must match the window listed in capabilities/default.json.
const MAIN_WINDOW_LABEL: &str = "main";

// Centres the window buttons in the 52px toolbar (h-13 in Toolbar.svelte).
// Measured: macOS offsets the buttons from this point, so it isn't (52 - 14) / 2.
#[cfg(target_os = "macos")]
const TRAFFIC_LIGHT_POSITION: (f64, f64) = (18.0, 27.0);

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // Must be registered first. A second launch focuses the open window instead
        // of starting another copy, which would clear this one's scratch folder.
        .plugin(tauri_plugin_single_instance::init(
            |app, _arguments, _working_directory| {
                focus_main_window(app);
            },
        ))
        .plugin(
            // Logs go to the per-user log folder (e.g. ~/Library/Logs/<app id>
            // on macOS, %LOCALAPPDATA%\<app id>\logs on Windows).
            tauri_plugin_log::Builder::new()
                .targets([
                    Target::new(TargetKind::LogDir { file_name: None }),
                    Target::new(TargetKind::Stdout),
                ])
                .level(log::LevelFilter::Info)
                .build(),
        )
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .setup(|app| {
            let app_state = create_app_state(app);
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

// Built in code so the macOS title bar options don't need a second config.
fn create_main_window(app: &App) -> tauri::Result<()> {
    let window_builder = WebviewWindowBuilder::new(app, MAIN_WINDOW_LABEL, WebviewUrl::default())
        .title("Converter")
        .inner_size(1080.0, 720.0)
        .min_inner_size(760.0, 540.0);

    // The app's toolbar sits under the window buttons, like native macOS apps.
    #[cfg(target_os = "macos")]
    let window_builder = window_builder
        .title_bar_style(tauri::TitleBarStyle::Overlay)
        .hidden_title(true)
        .traffic_light_position(tauri::LogicalPosition::new(
            TRAFFIC_LIGHT_POSITION.0,
            TRAFFIC_LIGHT_POSITION.1,
        ));

    window_builder.build()?;
    Ok(())
}

fn create_app_state(app: &App) -> AppState {
    let ffmpeg = Ffmpeg::locate().map_err(|error| error.to_string());
    if let Err(message) = &ffmpeg {
        log::error!("{message}");
    }

    // Without a cache folder, scratch files go in the system's temporary folder.
    let cache_directory = app
        .path()
        .app_cache_dir()
        .unwrap_or_else(|_| std::env::temp_dir());

    let app_state = AppState {
        ffmpeg,
        scratch_root: scratch::prepare_scratch_root(&cache_directory),
        active_batches: ActiveBatches::default(),
        produced_outputs: Arc::new(ProducedOutputs::default()),
    };
    app_state
}

fn focus_main_window(app: &AppHandle) {
    let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL) else {
        return;
    };
    let _ = window.unminimize();
    let _ = window.set_focus();
}
