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

#[cfg(target_os = "macos")]
pub use macos_menu::{build_macos_menu, handle_menu_event};

#[cfg(target_os = "macos")]
mod macos_menu {
    use tauri::menu::{Menu, MenuEvent, MenuItem};
    use tauri::{AppHandle, Emitter, Manager, Wry};

    use crate::state::AppState;
    use crate::{MAIN_WINDOW_LABEL, focus_main_window};

    // Sent to the main window when the user quits from the app menu while a
    // conversion is running. The UI asks first, as it does when the window closes.
    const QUIT_REQUESTED_EVENT: &str = "quit-requested";
    const QUIT_MENU_ITEM_ID: &str = "quit";

    // The default menu's Quit item (⌘Q) ends the app straight away, skipping the
    // window's close guard. It's replaced with one that asks first.
    pub fn build_macos_menu(app: &AppHandle) -> tauri::Result<Menu<Wry>> {
        let menu = Menu::default(app)?;
        let app_submenu = menu
            .items()?
            .first()
            .and_then(|item| item.as_submenu().cloned());
        let Some(app_submenu) = app_submenu else {
            return Ok(menu);
        };

        let app_submenu_items = app_submenu.items()?;
        let is_last_item_predefined = app_submenu_items
            .last()
            .is_some_and(|item| item.as_predefined_menuitem().is_some());
        if !is_last_item_predefined {
            return Ok(menu);
        }

        app_submenu.remove_at(app_submenu_items.len() - 1)?;
        let quit_label = format!("Quit {}", app.package_info().name);
        let quit_item = MenuItem::with_id(app, QUIT_MENU_ITEM_ID, quit_label, true, Some("Cmd+Q"))?;
        app_submenu.append(&quit_item)?;
        Ok(menu)
    }

    #[allow(
        clippy::needless_pass_by_value,
        reason = "Tauri passes menu events by value"
    )]
    pub fn handle_menu_event(app: &AppHandle, event: MenuEvent) {
        if event.id() == QUIT_MENU_ITEM_ID {
            request_quit(app);
        }
    }

    fn request_quit(app: &AppHandle) {
        let is_converting = app
            .try_state::<AppState>()
            .is_some_and(|state| state.active_batches.is_any_running());
        let has_main_window = app.get_webview_window(MAIN_WINDOW_LABEL).is_some();

        if !(is_converting && has_main_window) {
            app.exit(0);
            return;
        }

        focus_main_window(app);
        if let Err(error) = app.emit_to(MAIN_WINDOW_LABEL, QUIT_REQUESTED_EVENT, ()) {
            log::warn!("couldn't ask the window before quitting: {error}");
            app.exit(0);
        }
    }
}
