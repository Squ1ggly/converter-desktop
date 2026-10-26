// The macOS menu bar: the default menu, with a Quit item that asks first
// mid-conversion and a Help item that opens the app's own Help page.
use tauri::menu::{HELP_SUBMENU_ID, Menu, MenuEvent, MenuItem, Submenu};
use tauri::{AppHandle, Emitter, Wry};

use crate::{MAIN_WINDOW_LABEL, focus_main_window, quit};

const QUIT_MENU_ITEM_ID: &str = "quit";
const HELP_MENU_ITEM_ID: &str = "help";
// Sent to the main window to open its Help page.
const HELP_REQUESTED_EVENT: &str = "help-requested";

pub fn build_macos_menu(app: &AppHandle) -> tauri::Result<Menu<Wry>> {
    let menu = Menu::default(app)?;
    replace_quit_item(app, &menu)?;
    add_help_item(app, &menu)?;
    Ok(menu)
}

// The default Quit item (⌘Q) ends the app straight away, skipping the window's
// close guard. It's replaced with one that asks first.
fn replace_quit_item(app: &AppHandle, menu: &Menu<Wry>) -> tauri::Result<()> {
    let app_submenu = menu
        .items()?
        .first()
        .and_then(|item| item.as_submenu().cloned());
    let Some(app_submenu) = app_submenu else {
        return Ok(());
    };

    let app_submenu_items = app_submenu.items()?;
    let is_last_item_predefined = app_submenu_items
        .last()
        .is_some_and(|item| item.as_predefined_menuitem().is_some());
    if !is_last_item_predefined {
        return Ok(());
    }

    app_submenu.remove_at(app_submenu_items.len() - 1)?;
    let quit_label = format!("Quit {}", app.package_info().name);
    let quit_item = MenuItem::with_id(app, QUIT_MENU_ITEM_ID, quit_label, true, Some("Cmd+Q"))?;
    app_submenu.append(&quit_item)?;
    Ok(())
}

// macOS gives the Help menu a search field and binds ⌘? to it, so the app's
// Help page is offered there rather than through its own ⌘? shortcut.
fn add_help_item(app: &AppHandle, menu: &Menu<Wry>) -> tauri::Result<()> {
    let help_submenu: Option<Submenu<Wry>> = menu
        .get(HELP_SUBMENU_ID)
        .and_then(|item| item.as_submenu().cloned());
    let Some(help_submenu) = help_submenu else {
        return Ok(());
    };

    let help_label = format!("{} Help", app.package_info().name);
    let help_item = MenuItem::with_id(app, HELP_MENU_ITEM_ID, help_label, true, None::<&str>)?;
    help_submenu.append(&help_item)?;
    Ok(())
}

#[allow(
    clippy::needless_pass_by_value,
    reason = "Tauri passes menu events by value"
)]
pub fn handle_menu_event(app: &AppHandle, event: MenuEvent) {
    if event.id() == QUIT_MENU_ITEM_ID {
        quit::request_quit(app);
        return;
    }
    if event.id() == HELP_MENU_ITEM_ID {
        open_help(app);
    }
}

fn open_help(app: &AppHandle) {
    focus_main_window(app);
    if let Err(error) = app.emit_to(MAIN_WINDOW_LABEL, HELP_REQUESTED_EVENT, ()) {
        log::warn!("couldn't open Help: {error}");
    }
}
