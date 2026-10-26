use std::time::Duration;

use tauri::{AppHandle, Manager, RunEvent};

use crate::state::AppState;

// Long enough for FFmpeg to stop and for partial outputs to be deleted.
const EXIT_CLEANUP_TIMEOUT: Duration = Duration::from_secs(3);

// The process ends after this returns, without running destructors, so child
// processes would keep running and partial outputs would stay behind.
#[allow(
    clippy::needless_pass_by_value,
    reason = "Tauri's event loop passes events by value"
)]
pub fn handle_run_event(app: &AppHandle, event: RunEvent) {
    if !matches!(event, RunEvent::Exit) {
        return;
    }
    let Some(state) = app.try_state::<AppState>() else {
        return;
    };
    if !state.active_batches.is_any_running() {
        return;
    }

    state.active_batches.cancel_all();
    let has_finished = state
        .active_batches
        .wait_until_finished(EXIT_CLEANUP_TIMEOUT);
    if !has_finished {
        log::warn!("quit before every conversion had stopped");
    }
}

// Sent to the main window when the user quits from the app menu while a
// conversion is running. The UI asks first, as it does when the window closes.
#[cfg(target_os = "macos")]
const QUIT_REQUESTED_EVENT: &str = "quit-requested";

// Quits straight away unless files are converting, in which case the window
// is asked to confirm first.
#[cfg(target_os = "macos")]
pub fn request_quit(app: &AppHandle) {
    use tauri::Emitter;

    let is_converting = app
        .try_state::<AppState>()
        .is_some_and(|state| state.active_batches.is_any_running());
    let has_main_window = app.get_webview_window(crate::MAIN_WINDOW_LABEL).is_some();

    if !(is_converting && has_main_window) {
        app.exit(0);
        return;
    }

    crate::focus_main_window(app);
    if let Err(error) = app.emit_to(crate::MAIN_WINDOW_LABEL, QUIT_REQUESTED_EVENT, ()) {
        log::warn!("couldn't ask the window before quitting: {error}");
        app.exit(0);
    }
}
