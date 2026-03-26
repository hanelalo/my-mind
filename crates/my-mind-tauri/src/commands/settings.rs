use crate::state::AppState;
use my_mind_core::config::AppConfig;
use tauri::State;
use tracing::info;

#[tauri::command]
pub async fn get_config(state: State<'_, AppState>) -> Result<AppConfig, String> {
    let config = state.config.lock().await;
    Ok(config.clone())
}

#[tauri::command]
pub async fn save_config(
    config: AppConfig,
    state: State<'_, AppState>,
) -> Result<(), String> {
    config.save().map_err(|e| e.to_string())?;
    *state.config.lock().await = config;
    info!("Config saved");
    Ok(())
}
