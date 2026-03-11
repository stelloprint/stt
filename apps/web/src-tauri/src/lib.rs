mod db;
mod keys;
mod permissions;
mod prefs;
mod stt;
mod type_;

use db::{Database, Entry, EntryCreate, Session, SessionCreate};
use permissions::{PermissionState, Permissions};
use prefs::{ModelProfile, Preferences, Prefs};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::sync::Arc;
use stt::{get_model_info, SttEngine, TranscriptionResult};
use type_::{ContextHeuristic, TypeMethod, TypeOptions, Typer};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelStatus {
    pub profile: String,
    pub filename: String,
    pub expected_sha256: String,
    pub file_exists: bool,
    pub computed_sha256: Option<String>,
    pub is_verified: bool,
}

fn compute_file_sha256(path: &std::path::Path) -> Result<String, String> {
    let data = std::fs::read(path).map_err(|e| e.to_string())?;
    let mut hasher = Sha256::new();
    hasher.update(&data);
    let result = hasher.finalize();
    Ok(hex::encode(result))
}

pub struct AppState {
    pub prefs: Arc<Prefs>,
    pub db: Arc<Database>,
    pub stt: Arc<SttEngine>,
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
fn get_model_statuses(_state: tauri::State<'_, AppState>) -> Result<Vec<ModelStatus>, String> {
    let models_dir = Prefs::get_models_dir().map_err(|e| e.to_string())?;

    let profiles = [
        (ModelProfile::EnglishSmall, "small.en"),
        (ModelProfile::MultilingualSmall, "multilingual-small"),
        (ModelProfile::MultilingualMedium, "multilingual-medium"),
    ];

    let mut statuses = Vec::new();
    for (profile, profile_name) in profiles {
        let model_info = get_model_info(&profile);
        let model_path = models_dir.join(&model_info.filename);
        let file_exists = model_path.exists();

        let (computed_sha256, is_verified) = if file_exists {
            let computed = compute_file_sha256(&model_path).ok();
            let verified = computed
                .as_ref()
                .map(|h| h == &model_info.sha256)
                .unwrap_or(false);
            (computed, verified)
        } else {
            (None, false)
        };

        statuses.push(ModelStatus {
            profile: profile_name.to_string(),
            filename: model_info.filename,
            expected_sha256: model_info.sha256,
            file_exists,
            computed_sha256,
            is_verified,
        });
    }

    Ok(statuses)
}

#[tauri::command]
fn get_current_model(_state: tauri::State<'_, AppState>) -> Option<String> {
    None
}

#[tauri::command]
fn verify_model(
    _state: tauri::State<'_, AppState>,
    profile: String,
) -> Result<ModelStatus, String> {
    let models_dir = Prefs::get_models_dir().map_err(|e| e.to_string())?;

    let model_profile = match profile.as_str() {
        "small.en" => ModelProfile::EnglishSmall,
        "multilingual-small" => ModelProfile::MultilingualSmall,
        "multilingual-medium" => ModelProfile::MultilingualMedium,
        _ => return Err(format!("Unknown profile: {}", profile)),
    };

    let model_info = get_model_info(&model_profile);
    let model_path = models_dir.join(&model_info.filename);
    let file_exists = model_path.exists();

    let (computed_sha256, is_verified) = if file_exists {
        let computed = compute_file_sha256(&model_path).map_err(|e| e.to_string())?;
        let verified = computed == model_info.sha256;
        (Some(computed), verified)
    } else {
        (None, false)
    };

    Ok(ModelStatus {
        profile,
        filename: model_info.filename,
        expected_sha256: model_info.sha256,
        file_exists,
        computed_sha256,
        is_verified,
    })
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

#[tauri::command]
fn load_model(state: tauri::State<'_, AppState>, profile: String) -> Result<(), String> {
    let profile = match profile.as_str() {
        "small.en" => prefs::ModelProfile::EnglishSmall,
        "multilingual-small" => prefs::ModelProfile::MultilingualSmall,
        "multilingual-medium" => prefs::ModelProfile::MultilingualMedium,
        _ => return Err(format!("Unknown profile: {}", profile)),
    };

    let models_dir = Prefs::get_models_dir().map_err(|e| e.to_string())?;
    state
        .stt
        .load_model(profile, models_dir)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn transcribe(
    state: tauri::State<'_, AppState>,
    audio_data: Vec<f32>,
) -> Result<TranscriptionResult, String> {
    let prefs = state.prefs.get();
    state
        .stt
        .transcribe(&audio_data, &prefs)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn is_model_loaded(state: tauri::State<'_, AppState>) -> bool {
    state.stt.is_loaded()
}

#[tauri::command]
fn check_microphone_permission() -> PermissionState {
    Permissions::check_microphone()
}

#[tauri::command]
fn check_accessibility_permission() -> PermissionState {
    Permissions::check_accessibility()
}

#[tauri::command]
fn request_microphone_permission() -> Result<PermissionState, String> {
    Permissions::request_microphone()
}

#[tauri::command]
fn request_accessibility_permission() -> Result<PermissionState, String> {
    Permissions::request_accessibility()
}

#[tauri::command]
fn open_microphone_settings() -> Result<(), String> {
    Permissions::open_microphone_settings()
}

#[tauri::command]
fn open_accessibility_settings() -> Result<(), String> {
    Permissions::open_accessibility_settings()
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

    let app_state = AppState {
        prefs,
        db,
        stt: Arc::new(SttEngine::new()),
    };

    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            get_preferences,
            update_preferences,
            get_config_dir,
            get_data_dir,
            get_models_dir,
            get_model_statuses,
            get_current_model,
            verify_model,
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
            load_model,
            transcribe,
            is_model_loaded,
            check_microphone_permission,
            check_accessibility_permission,
            request_microphone_permission,
            request_accessibility_permission,
            open_microphone_settings,
            open_accessibility_settings,
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
