use my_mind_core::config::AppConfig;
use my_mind_core::history::HistoryStore;
use tokio::sync::{mpsc, Mutex};

/// Shared application state managed by Tauri
pub struct AppState {
    pub config: Mutex<AppConfig>,
    pub pipeline_stop_tx: Mutex<Option<mpsc::Sender<()>>>,
    pub is_recording: Mutex<bool>,
    /// Bundle identifier of the app that was frontmost before overlay was shown.
    /// Uses std::sync::Mutex because it's accessed in both sync (shortcut handler)
    /// and async contexts, and only ever held briefly.
    pub previous_app: std::sync::Mutex<Option<String>>,
    pub history: HistoryStore,
}

impl AppState {
    pub fn new(config: AppConfig, history: HistoryStore) -> Self {
        Self {
            config: Mutex::new(config),
            pipeline_stop_tx: Mutex::new(None),
            is_recording: Mutex::new(false),
            previous_app: std::sync::Mutex::new(None),
            history,
        }
    }
}
