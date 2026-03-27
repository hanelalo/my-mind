use crate::state::AppState;
use my_mind_core::history::HistoryRecord;
use tauri::State;

#[tauri::command]
pub async fn get_history(
    state: State<'_, AppState>,
    limit: u32,
    offset: u32,
) -> Result<Vec<HistoryRecord>, String> {
    state.history.list(limit, offset).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_history_count(state: State<'_, AppState>) -> Result<u64, String> {
    state.history.count().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_history_record(
    state: State<'_, AppState>,
    id: String,
) -> Result<(), String> {
    state.history.delete(&id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn clear_history(state: State<'_, AppState>) -> Result<(), String> {
    state.history.clear().map_err(|e| e.to_string())
}
