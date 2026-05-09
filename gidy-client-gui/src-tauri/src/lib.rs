mod commands;

use commands::ProxyState;
use parking_lot::Mutex;
use std::sync::Arc;

pub struct AppState {
    pub proxy: Arc<Mutex<ProxyState>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            proxy: Arc::new(Mutex::new(ProxyState::default())),
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let state = AppState::new();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            commands::connect,
            commands::disconnect,
            commands::get_stats,
            commands::get_config,
            commands::update_config,
            commands::get_status,
            commands::generate_psk,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
