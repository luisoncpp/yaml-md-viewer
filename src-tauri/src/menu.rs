use crate::{commands::apply_fullscreen, models::AppState};
use std::sync::Mutex;
use tauri::{
    AppHandle, Emitter, Manager,
    menu::{Menu, MenuBuilder, MenuItem, MenuItemBuilder, MenuItemKind, SubmenuBuilder},
};

pub const OPEN: &str = "open-document";
pub const SAVE: &str = "save-as-html";
pub const FULLSCREEN: &str = "fullscreen";
pub const EXIT_FULLSCREEN: &str = "exit-fullscreen";
pub const ZOOM_IN: &str = "zoom-in";
pub const ZOOM_OUT: &str = "zoom-out";
pub const ZOOM_RESET: &str = "zoom-reset";

pub fn build(app: &AppHandle) -> tauri::Result<Menu<tauri::Wry>> {
    let open = MenuItemBuilder::with_id(OPEN, "&Open…")
        .accelerator("Ctrl+O")
        .build(app)?;
    let save = MenuItemBuilder::with_id(SAVE, "&Save as HTML…")
        .accelerator("Ctrl+Shift+S")
        .enabled(false)
        .build(app)?;
    let fullscreen = MenuItemBuilder::with_id(FULLSCREEN, "&Fullscreen").build(app)?;
    let exit_fullscreen = MenuItemBuilder::with_id(EXIT_FULLSCREEN, "E&xit Fullscreen")
        .enabled(false)
        .build(app)?;
    let zoom_in = MenuItemBuilder::with_id(ZOOM_IN, "Zoom &In")
        .accelerator("Ctrl+=")
        .build(app)?;
    let zoom_out = MenuItemBuilder::with_id(ZOOM_OUT, "Zoom &Out")
        .accelerator("Ctrl+-")
        .build(app)?;
    let zoom_reset = MenuItemBuilder::with_id(ZOOM_RESET, "&Reset Zoom (100%)")
        .accelerator("Ctrl+0")
        .build(app)?;

    let file = SubmenuBuilder::new(app, "&File")
        .item(&open)
        .item(&save)
        .separator()
        .quit()
        .build()?;
    let view = SubmenuBuilder::new(app, "&View")
        .item(&zoom_in)
        .item(&zoom_out)
        .item(&zoom_reset)
        .separator()
        .item(&fullscreen)
        .item(&exit_fullscreen)
        .build()?;

    MenuBuilder::new(app).item(&file).item(&view).build()
}

pub fn handle_event(app: &AppHandle, id: &str) {
    match id {
        OPEN => {
            let _ = app.emit("menu-open-document", ());
        }
        SAVE => {
            let _ = app.emit("menu-save-as-html", ());
        }
        ZOOM_IN => {
            let _ = app.emit("menu-zoom-in", ());
        }
        ZOOM_OUT => {
            let _ = app.emit("menu-zoom-out", ());
        }
        ZOOM_RESET => {
            let _ = app.emit("menu-zoom-reset", ());
        }
        FULLSCREEN => toggle_fullscreen(app),
        EXIT_FULLSCREEN => set_fullscreen(app, false),
        _ => {}
    }
}

pub fn set_save_enabled(app: &AppHandle, enabled: bool) {
    if let Some(item) = find_item(app, SAVE) {
        let _ = item.set_enabled(enabled);
    }
}

pub fn sync_fullscreen_items(app: &AppHandle, fullscreen: bool) {
    if let Some(item) = find_item(app, FULLSCREEN) {
        let _ = item.set_text(if fullscreen {
            "E&xit Fullscreen"
        } else {
            "&Fullscreen"
        });
    }
    if let Some(item) = find_item(app, EXIT_FULLSCREEN) {
        let _ = item.set_enabled(fullscreen);
    }
}

pub fn sync_zoom_item(app: &AppHandle, percentage: i32) {
    if let Some(item) = find_item(app, ZOOM_RESET) {
        let _ = item.set_text(format!("&Reset Zoom ({percentage}%)"));
    }
}

fn toggle_fullscreen(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let fullscreen = window.is_fullscreen().unwrap_or(false);
        if let Err(error) = apply_fullscreen(&window, !fullscreen) {
            eprintln!("Fullscreen menu action: {error}");
        }
    }
}

fn set_fullscreen(app: &AppHandle, fullscreen: bool) {
    if let Some(window) = app.get_webview_window("main")
        && let Err(error) = apply_fullscreen(&window, fullscreen)
    {
        eprintln!("Fullscreen menu action: {error}");
    }
}

fn find_item(app: &AppHandle, id: &str) -> Option<MenuItem<tauri::Wry>> {
    app.menu()?.items().ok()?.into_iter().find_map(|item| {
        let MenuItemKind::Submenu(submenu) = item else {
            return None;
        };
        match submenu.get(id) {
            Some(MenuItemKind::MenuItem(item)) => Some(item),
            _ => None,
        }
    })
}

pub fn has_document(app: &AppHandle) -> bool {
    app.state::<Mutex<AppState>>()
        .lock()
        .map(|state| state.current.is_some())
        .unwrap_or(false)
}
