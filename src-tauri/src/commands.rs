use crate::aggregator::{aggregate_to_kline, period_to_seconds, KLineData};
use crate::database::{DatabaseManager, SystemMetrics};
use std::sync::{Arc, Mutex};
use tauri::State;

pub struct AppState {
    pub db_manager: Arc<Mutex<DatabaseManager>>,
}

#[tauri::command]
pub async fn get_historical_data(
    indicator: String,
    period: String,
    start_time: i64,
    end_time: i64,
    state: State<'_, AppState>,
) -> Result<Vec<KLineData>, String> {
    let db = state.db_manager.lock().map_err(|e| e.to_string())?;

    let raw_data = db
        .query_raw_data(start_time, end_time)
        .map_err(|e| e.to_string())?;

    let period_seconds = period_to_seconds(&period)?;

    Ok(aggregate_to_kline(raw_data, period_seconds, &indicator))
}

#[tauri::command]
pub async fn get_latest_data(
    state: State<'_, AppState>,
) -> Result<Option<SystemMetrics>, String> {
    let db = state.db_manager.lock().map_err(|e| e.to_string())?;
    db.get_latest_data().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn start_collection() -> Result<(), String> {
    // Collection is started automatically in main.rs setup
    // This command is kept for compatibility but does nothing
    Ok(())
}
