use crate::{
    document::DocumentService,
    error::AppError,
    models::{AppState, DocumentView, SaveResult},
};
use std::{path::PathBuf, sync::Mutex};
use tauri::{AppHandle, Emitter, Manager, State, WebviewWindow};

pub fn open_and_store(path: PathBuf, state: &Mutex<AppState>) -> Result<DocumentView, AppError> {
    let mut snapshot = DocumentService::compile_path(&path)?;
    let mut state = state
        .lock()
        .map_err(|_| AppError::new("read_failed", "The application state is unavailable."))?;
    state.next_revision += 1;
    snapshot.revision = state.next_revision;
    let view = DocumentView::from(&snapshot);
    state.current = Some(snapshot);
    Ok(view)
}

#[tauri::command]
pub fn open_document(
    path: String,
    state: State<'_, Mutex<AppState>>,
    app: AppHandle,
) -> Result<DocumentView, AppError> {
    let document = open_and_store(PathBuf::from(path), &state)?;
    crate::menu::set_save_enabled(&app, true);
    Ok(document)
}

#[tauri::command]
pub fn get_current_document(
    state: State<'_, Mutex<AppState>>,
) -> Result<Option<DocumentView>, AppError> {
    let state = state
        .lock()
        .map_err(|_| AppError::new("read_failed", "The application state is unavailable."))?;
    Ok(state.current.as_ref().map(DocumentView::from))
}

#[tauri::command]
pub fn save_current_document(
    path: String,
    state: State<'_, Mutex<AppState>>,
) -> Result<SaveResult, AppError> {
    let html = {
        let state = state
            .lock()
            .map_err(|_| AppError::new("read_failed", "The application state is unavailable."))?;
        state
            .current
            .as_ref()
            .map(|document| document.compiled_html.clone())
            .ok_or_else(|| AppError::new("no_document", "Open a document before saving."))?
    };
    let path = DocumentService::export(&PathBuf::from(path), &html)?;
    Ok(SaveResult {
        path: path.to_string_lossy().into_owned(),
    })
}

#[tauri::command]
pub fn set_fullscreen(window: WebviewWindow, fullscreen: bool) -> Result<bool, AppError> {
    apply_fullscreen(&window, fullscreen)
}

pub fn apply_fullscreen(window: &WebviewWindow, fullscreen: bool) -> Result<bool, AppError> {
    window
        .set_fullscreen(fullscreen)
        .map_err(|_| AppError::new("write_failed", "Fullscreen could not be changed."))?;
    let fullscreen = window
        .is_fullscreen()
        .map_err(|_| AppError::new("write_failed", "Fullscreen state could not be read."))?;
    if fullscreen {
        window.hide_menu()
    } else {
        window.show_menu()
    }
    .map_err(|_| {
        AppError::new(
            "write_failed",
            "The menu bar visibility could not be changed.",
        )
    })?;
    crate::menu::sync_fullscreen_items(window.app_handle(), fullscreen);
    let _ = window.emit("fullscreen-changed", fullscreen);
    Ok(fullscreen)
}

#[tauri::command]
pub fn get_fullscreen(window: WebviewWindow) -> Result<bool, AppError> {
    window
        .is_fullscreen()
        .map_err(|_| AppError::new("write_failed", "Fullscreen state could not be read."))
}

#[tauri::command]
pub fn set_zoom(window: WebviewWindow, percentage: i32) -> Result<i32, AppError> {
    let percentage = percentage.clamp(25, 500);
    window
        .set_zoom(f64::from(percentage) / 100.0)
        .map_err(|_| AppError::new("write_failed", "Zoom could not be changed."))?;
    crate::menu::sync_zoom_item(window.app_handle(), percentage);
    Ok(percentage)
}
