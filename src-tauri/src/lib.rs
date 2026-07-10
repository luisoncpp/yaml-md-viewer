mod arguments;
mod commands;
mod document;
mod error;
mod instance;
mod menu;
mod models;
mod preview;

use crate::{
    arguments::parse_document_argument,
    commands::{
        get_current_document, get_fullscreen, open_document, save_current_document, set_fullscreen,
        set_zoom,
    },
    instance::{handle_secondary_instance, load_path},
    models::AppState,
};
use std::{path::PathBuf, sync::Mutex};

pub fn run() {
    tauri::Builder::default()
        .register_uri_scheme_protocol("yamlmdpreview", |context, request| {
            preview::respond(context.app_handle(), request.uri().path())
        })
        .plugin(tauri_plugin_single_instance::init(|app, argv, cwd| {
            handle_secondary_instance(app.clone(), argv, cwd)
        }))
        .plugin(tauri_plugin_dialog::init())
        .manage(Mutex::new(AppState::default()))
        .menu(menu::build)
        .on_menu_event(|app, event| menu::handle_event(app, event.id().as_ref()))
        .setup(|app| {
            if let Some(path) = parse_document_argument(
                std::env::args_os(),
                &std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            )? && let Err(error) = load_path(app.handle(), path)
            {
                eprintln!("Startup document: {error}");
            }
            menu::set_save_enabled(app.handle(), menu::has_document(app.handle()));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            open_document,
            get_current_document,
            save_current_document,
            set_fullscreen,
            get_fullscreen,
            set_zoom
        ])
        .run(tauri::generate_context!())
        .expect("error while running YAML Markdown Viewer");
}
