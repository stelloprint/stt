mod prefs;

use prefs::{Preferences, Prefs};
use std::sync::Arc;

pub struct AppState {
    pub prefs: Arc<Prefs>,
}

#[tauri::command]
fn get_preferences(state: tauri::State<'_, AppState>) -> Preferences {
    state.prefs.get()
}

#[tauri::command]
fn update_preferences(state: tauri::State<'_, AppState>, prefs: Preferences) -> Result<(), String> {
    state.prefs.update(prefs).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_config_dir(_state: tauri::State<'_, AppState>) -> Result<String, String> {
    Prefs::get_config_dir()
        .map(|p| p.to_string_lossy().to_string())
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn get_data_dir(_state: tauri::State<'_, AppState>) -> Result<String, String> {
    Prefs::get_data_dir()
        .map(|p| p.to_string_lossy().to_string())
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn get_models_dir(_state: tauri::State<'_, AppState>) -> Result<String, String> {
    Prefs::get_models_dir()
        .map(|p| p.to_string_lossy().to_string())
        .map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let prefs = match Prefs::new() {
        Ok(p) => Arc::new(p),
        Err(e) => {
            log::error!("Failed to initialize preferences: {}", e);
            panic!("Failed to initialize preferences: {}", e);
        }
    };

    let app_state = AppState { prefs };

    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            get_preferences,
            update_preferences,
            get_config_dir,
            get_data_dir,
            get_models_dir,
        ])
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            log::info!("STT App initialized");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
