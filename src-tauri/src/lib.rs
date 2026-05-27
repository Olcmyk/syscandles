mod aggregator;
mod collector;
mod commands;
mod database;

use collector::DataCollector;
use commands::AppState;
use database::DatabaseManager;
use std::sync::{Arc, Mutex};
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // Get app data directory
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("Failed to get app data dir");

            // Create directory if it doesn't exist
            std::fs::create_dir_all(&app_data_dir)?;

            let db_path = app_data_dir.join("data.db");

            // Initialize database
            let db_manager = Arc::new(Mutex::new(
                DatabaseManager::new(&db_path).expect("Failed to initialize database"),
            ));

            // Initialize collector
            let collector = Arc::new(DataCollector::new(db_manager.clone()));

            // Start collection loop in background using Tauri's async runtime
            let app_handle = app.handle().clone();
            let collector_clone = collector.clone();

            tauri::async_runtime::spawn(async move {
                collector_clone.start_collection_loop(app_handle).await;
            });

            // Manage state
            app.manage(AppState { db_manager });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_historical_data,
            commands::get_latest_data,
            commands::start_collection,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
