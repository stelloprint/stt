mod db;
mod prefs;

use db::{Database, Entry, EntryCreate, Session, SessionCreate};
use prefs::{Preferences, Prefs};
use std::sync::Arc;

pub struct AppState {
    pub prefs: Arc<Prefs>,
    pub db: Arc<Database>,
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

#[tauri::command]
fn create_session(
    state: tauri::State<'_, AppState>,
    session: SessionCreate,
) -> Result<Session, String> {
    state.db.create_session(session).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_session(state: tauri::State<'_, AppState>, id: String) -> Result<Option<Session>, String> {
    state.db.get_session(&id).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_all_sessions(state: tauri::State<'_, AppState>) -> Result<Vec<Session>, String> {
    state.db.get_all_sessions().map_err(|e| e.to_string())
}

#[tauri::command]
fn update_session(
    state: tauri::State<'_, AppState>,
    id: String,
    ended_at: Option<i64>,
    chars_count: Option<i64>,
    words_count: Option<i64>,
) -> Result<Option<Session>, String> {
    state
        .db
        .update_session(&id, ended_at, chars_count, words_count)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_session(state: tauri::State<'_, AppState>, id: String) -> Result<bool, String> {
    state.db.delete_session(&id).map_err(|e| e.to_string())
}

#[tauri::command]
fn create_entry(state: tauri::State<'_, AppState>, entry: EntryCreate) -> Result<Entry, String> {
    state.db.create_entry(entry).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_entry(state: tauri::State<'_, AppState>, id: String) -> Result<Option<Entry>, String> {
    state.db.get_entry(&id).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_entries_by_session(
    state: tauri::State<'_, AppState>,
    session_id: String,
) -> Result<Vec<Entry>, String> {
    state
        .db
        .get_entries_by_session(&session_id)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn get_all_entries(state: tauri::State<'_, AppState>) -> Result<Vec<Entry>, String> {
    state.db.get_all_entries().map_err(|e| e.to_string())
}

#[tauri::command]
fn update_entry(
    state: tauri::State<'_, AppState>,
    id: String,
    text: Option<String>,
    typed: Option<bool>,
) -> Result<Option<Entry>, String> {
    state
        .db
        .update_entry(&id, text, typed)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_entry(state: tauri::State<'_, AppState>, id: String) -> Result<bool, String> {
    state.db.delete_entry(&id).map_err(|e| e.to_string())
}

#[tauri::command]
fn search_entries(state: tauri::State<'_, AppState>, query: String) -> Result<Vec<Entry>, String> {
    state.db.search_entries(&query).map_err(|e| e.to_string())
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

    let db = match Database::new() {
        Ok(d) => Arc::new(d),
        Err(e) => {
            log::error!("Failed to initialize database: {}", e);
            panic!("Failed to initialize database: {}", e);
        }
    };

    let app_state = AppState { prefs, db };

    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            get_preferences,
            update_preferences,
            get_config_dir,
            get_data_dir,
            get_models_dir,
            create_session,
            get_session,
            get_all_sessions,
            update_session,
            delete_session,
            create_entry,
            get_entry,
            get_entries_by_session,
            get_all_entries,
            update_entry,
            delete_entry,
            search_entries,
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
