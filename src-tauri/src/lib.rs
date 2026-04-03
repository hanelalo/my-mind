mod setup;

use my_mind_core::config::AppConfig;
use my_mind_core::history::HistoryStore;
use my_mind_tauri::commands::{diagnosis, history, recording, settings};
use my_mind_tauri::state::AppState;
use tracing::info;
use tracing_subscriber::EnvFilter;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    info!("Starting My Mind...");

    // Load config
    let config = AppConfig::load().unwrap_or_else(|e| {
        tracing::warn!("Failed to load config, using defaults: {}", e);
        AppConfig::default()
    });

    info!("Config loaded - ASR api_key present: {}, ASR base_url: {:?}, LLM provider: {}",
        !config.asr.online.api_key.is_empty(),
        config.asr.online.api_base_url,
        config.llm.provider,
    );

    let history = {
        let path = HistoryStore::default_path().expect("Cannot determine history database path");
        HistoryStore::new(path).expect("Failed to open history database")
    };
    let app_state = AppState::new(config, history);

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|_app, _args, _cwd| {}))
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            recording::start_recording,
            recording::stop_recording,
            recording::cancel_recording,
            recording::is_recording,
            settings::get_config,
            settings::save_config,
            history::get_history,
            history::get_history_count,
            history::delete_history_record,
            history::clear_history,
            diagnosis::diagnose_prompt,
        ])
        .setup(setup::setup)
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
