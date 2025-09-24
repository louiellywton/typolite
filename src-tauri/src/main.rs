// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::{CustomMenuItem, Menu, MenuItem, Submenu, WindowEvent};
use tracing::info;
use tracing_subscriber::EnvFilter;

mod parser;
mod export;
mod file_service;
mod commands;

use commands::*;
use crate::commands::AppState;

/// Initialize logging for the application
fn init_logging() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("typolite=debug".parse().unwrap()))
        .init();
}

/// Create application menu
fn create_menu() -> Menu {
    let quit = CustomMenuItem::new("quit".to_string(), "Quit");
    let close = CustomMenuItem::new("close".to_string(), "Close");
    let file_menu = Submenu::new("File", Menu::new().add_item(close).add_item(quit));
    
    let copy = CustomMenuItem::new("copy".to_string(), "Copy");
    let paste = CustomMenuItem::new("paste".to_string(), "Paste");
    let edit_menu = Submenu::new("Edit", Menu::new().add_item(copy).add_item(paste));
    
    Menu::new()
        .add_native_item(MenuItem::About("Typora-Lite".to_string(), Default::default()))
        .add_submenu(file_menu)
        .add_submenu(edit_menu)
}

fn main() {
    init_logging();
    info!("Starting Typora-Lite v{}", env!("CARGO_PKG_VERSION"));

    let app_state = AppState::default();

    tauri::Builder::default()
        .manage(app_state)
        .menu(create_menu())
        .on_menu_event(|event| {
            match event.menu_item_id() {
                "quit" => {
                    std::process::exit(0);
                }
                "close" => {
                    event.window().close().unwrap();
                }
                _ => {}
            }
        })
        .on_window_event(|event| match event.event() {
            WindowEvent::CloseRequested { api, .. } => {
                info!("Window close requested");
                api.prevent_close();
            }
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![
            open_file_dialog,
            read_markdown_file,
            parse_markdown,
            export_to_pdf,
            get_app_config_dir,
            save_file,
            watch_file,
            unwatch_file,
            get_file_metadata,
            list_recent_files,
            get_app_version,
            get_system_info
        ])
        .setup(|_app| {
            info!("Typora-Lite setup complete");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
