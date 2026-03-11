mod audio;
mod db;
mod keys;
mod permissions;
mod prefs;
mod stt;
mod type_;

use audio::AudioHandle;
use db::{Database, Entry, EntryCreate, Session, SessionCreate};
use keys::{ActivationState, KeysHandle};
use parking_lot::RwLock;
use permissions::{PermissionState, Permissions};
use prefs::{ActivationMode, Preferences, Prefs};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use stt::{SttEngine, TranscriptionResult};
use type_::{ContextHeuristic, TypeMethod, TypeOptions, Typer};

pub struct AppState {
    pub prefs: Arc<Prefs>,
    pub db: Arc<Database>,
    pub stt: Arc<SttEngine>,
    pub audio: Arc<AudioHandle>,
    pub keys: RwLock<Option<Arc<KeysHandle>>>,
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

    let audio = match AudioHandle::new() {
        Ok(a) => Arc::new(a),
        Err(e) => {
            log::error!("Failed to initialize audio: {}", e);
            panic!("Failed to initialize audio: {}", e);
        }
    };

    let stt_engine = Arc::new(SttEngine::new());

    let keys_handle = match KeysHandle::new() {
        Ok(k) => {
            let audio = Arc::clone(&audio);
            let stt = Arc::clone(&stt_engine);
            let prefs = Arc::clone(&prefs);
            let db = Arc::clone(&db);
            let toggle_active = Arc::new(AtomicBool::new(false));

            let toggle_active_clone = Arc::clone(&toggle_active);
            let audio_clone = Arc::clone(&audio);
            let prefs_clone = Arc::clone(&prefs);
            let stt_clone = Arc::clone(&stt_engine);

            thread::spawn(move || loop {
                thread::sleep(std::time::Duration::from_millis(100));

                if toggle_active_clone.load(Ordering::SeqCst) && audio_clone.is_recording() {
                    let silence_ms = audio_clone.get_silence_duration_ms();
                    let prefs = prefs_clone.get();
                    let silence_seconds = prefs.silence_seconds * 1000.0 as f32;

                    if silence_ms > silence_seconds as u64 && silence_ms > 0 {
                        log::info!("Silence timeout - stopping capture");
                        if let Err(e) = audio_clone.stop() {
                            log::error!("Failed to stop audio capture: {}", e);
                        }

                        let audio_data = audio_clone.get_buffer();
                        if !audio_data.is_empty() {
                            log::info!("Transcribing {} audio samples", audio_data.len());
                            match stt_clone.transcribe(&audio_data, &prefs) {
                                Ok(result) => {
                                    log::info!("Transcription result: {}", result.text);
                                    if !result.text.is_empty() {
                                        match Typer::with_defaults() {
                                            Ok(typer) => {
                                                if let Err(e) = typer.type_text(&result.text) {
                                                    log::error!("Failed to type text: {}", e);
                                                }
                                            }
                                            Err(e) => {
                                                log::error!("Failed to create typer: {}", e);
                                            }
                                        }
                                    }
                                    if let Err(e) = audio_clone.clear_buffer() {
                                        log::error!("Failed to clear audio buffer: {}", e);
                                    }
                                }
                                Err(e) => {
                                    log::error!("Transcription failed: {}", e);
                                }
                            }
                        }

                        toggle_active_clone.store(false, Ordering::SeqCst);
                    }
                }
            });

            let toggle_active_for_callback = Arc::clone(&toggle_active);

            k.on_activation(move |state, _source| {
                let prefs = prefs.get();
                let is_toggle = prefs.mode == ActivationMode::Toggle;
                let toggle_active = toggle_active_for_callback.load(Ordering::SeqCst);

                match state {
                    ActivationState::Active => {
                        if is_toggle {
                            if !toggle_active {
                                log::info!("Toggle mode: starting audio capture");
                                if let Err(e) = audio.start() {
                                    log::error!("Failed to start audio capture: {}", e);
                                }
                                toggle_active_for_callback.store(true, Ordering::SeqCst);
                            } else {
                                log::info!("Toggle mode: manual stop triggered");
                                if let Err(e) = audio.stop() {
                                    log::error!("Failed to stop audio capture: {}", e);
                                }

                                let audio_data = audio.get_buffer();
                                if !audio_data.is_empty() {
                                    log::info!("Transcribing {} audio samples", audio_data.len());
                                    match stt.transcribe(&audio_data, &prefs) {
                                        Ok(result) => {
                                            log::info!("Transcription result: {}", result.text);
                                            if !result.text.is_empty() {
                                                match Typer::with_defaults() {
                                                    Ok(typer) => {
                                                        if let Err(e) =
                                                            typer.type_text(&result.text)
                                                        {
                                                            log::error!(
                                                                "Failed to type text: {}",
                                                                e
                                                            );
                                                        }
                                                    }
                                                    Err(e) => {
                                                        log::error!(
                                                            "Failed to create typer: {}",
                                                            e
                                                        );
                                                    }
                                                }
                                            }
                                            if let Err(e) = audio.clear_buffer() {
                                                log::error!("Failed to clear audio buffer: {}", e);
                                            }
                                        }
                                        Err(e) => {
                                            log::error!("Transcription failed: {}", e);
                                        }
                                    }
                                }

                                toggle_active_for_callback.store(false, Ordering::SeqCst);
                            }
                        } else {
                            log::info!("Hold mode: starting audio capture");
                            if let Err(e) = audio.start() {
                                log::error!("Failed to start audio capture: {}", e);
                            }
                        }
                    }
                    ActivationState::Inactive => {
                        if !is_toggle {
                            log::info!("Hold mode: releasing - stopping capture");
                            if let Err(e) = audio.stop() {
                                log::error!("Failed to stop audio capture: {}", e);
                            }

                            let audio_data = audio.get_buffer();
                            if !audio_data.is_empty() {
                                log::info!("Transcribing {} audio samples", audio_data.len());
                                match stt.transcribe(&audio_data, &prefs) {
                                    Ok(result) => {
                                        log::info!("Transcription result: {}", result.text);
                                        if !result.text.is_empty() {
                                            match Typer::with_defaults() {
                                                Ok(typer) => {
                                                    if let Err(e) = typer.type_text(&result.text) {
                                                        log::error!("Failed to type text: {}", e);
                                                    }
                                                }
                                                Err(e) => {
                                                    log::error!("Failed to create typer: {}", e);
                                                }
                                            }
                                        }
                                        if let Err(e) = audio.clear_buffer() {
                                            log::error!("Failed to clear audio buffer: {}", e);
                                        }
                                    }
                                    Err(e) => {
                                        log::error!("Transcription failed: {}", e);
                                    }
                                }
                            }
                        }
                    }
                }
            });
            Some(Arc::new(k))
        }
        Err(e) => {
            log::error!("Failed to initialize keyboard listener: {:?}", e);
            None
        }
    };

    let app_state = AppState {
        prefs,
        db,
        stt: stt_engine,
        audio,
        keys: RwLock::new(keys_handle),
    };

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
