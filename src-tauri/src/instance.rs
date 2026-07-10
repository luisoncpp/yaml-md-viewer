use crate::{
    arguments::parse_document_argument,
    commands::open_and_store,
    error::AppError,
    models::{AppState, DocumentView},
};
use std::{ffi::OsString, path::PathBuf, sync::Mutex};
use tauri::{AppHandle, Emitter, Manager};

pub fn load_path(app: &AppHandle, path: PathBuf) -> Result<DocumentView, AppError> {
    let state = app.state::<Mutex<AppState>>();
    let document = open_and_store(path, &state)?;
    crate::menu::set_save_enabled(app, true);
    Ok(document)
}

pub fn handle_secondary_instance(app: AppHandle, arguments: Vec<String>, cwd: String) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
    std::thread::spawn(move || {
        let callback_cwd = PathBuf::from(cwd);
        let parsed =
            parse_document_argument(arguments.into_iter().map(OsString::from), &callback_cwd);
        let result = parsed.and_then(|path| path.map(|path| load_path(&app, path)).transpose());
        match result {
            Ok(Some(document)) => {
                let _ = app.emit("document-opened", document);
            }
            Ok(None) => {}
            Err(error) => {
                eprintln!("Secondary invocation: {error}");
                let _ = app.emit("document-error", error);
            }
        }
    });
}
