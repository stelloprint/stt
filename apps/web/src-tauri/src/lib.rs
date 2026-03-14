mod audio;
mod db;
mod keys;
mod permissions;
mod prefs;
mod session;
mod stt;
mod type_;

use audio::{AudioHandle, RecordCapture, SilenceLevel};
use db::{Database, Entry, EntryCreate, Session, SessionCreate};
use keys::{ActivationState, KeysHandle};
use parking_lot::Mutex;
use parking_lot::RwLock;
use permissions::{PermissionState, Permissions};
use prefs::{Preferences, Prefs};
use session::SessionManager;
use std::sync::Arc;
use stt::{SttEngine, TranscriptionResult};

pub struct AppState {
    pub prefs: Arc<Prefs>,
    pub db: Arc<Database>,
    pub stt: Arc<SttEngine>,
    pub session_manager: Arc<SessionManager>,
    pub audio: Arc<AudioHandle>,
    pub record_capture: Arc<Mutex<Option<RecordCapture>>>,
    pub keys: RwLock<Option<Arc<KeysHandle>>>,
}

#[tauri::command]
fn get_preferences(state: tauri::State<'_, AppState>) -> Preferences {
    state.prefs.get()
}

#[tauri::command]
fn update_preferences(state: tauri::State<'_, AppState>, prefs: Preferences) -> Result<(), String> {
    state
        .prefs
        .update(prefs.clone())
        .map_err(|e| e.to_string())?;

    state
        .audio
        .set_silence_threshold(silence_level_from_pref(&prefs.silence_rms));

    if let Some(keys) = state.keys.read().as_ref() {
        keys.set_enabled(prefs.hotkeys.left_chord, prefs.hotkeys.right_chord);
    }

    Ok(())
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

#[tauri::command]
fn get_frontmost_app_name() -> Option<String> {
    get_frontmost_app_name_internal()
}

#[tauri::command]
fn start_session(state: tauri::State<'_, AppState>, mode: String) -> Result<Session, String> {
    let session_mode = match mode.as_str() {
        "hold" => db::SessionMode::Hold,
        "toggle" => db::SessionMode::Toggle,
        "record" => db::SessionMode::Record,
        _ => return Err(format!("Unknown mode: {}", mode)),
    };
    session::create_session_workflow(&state, session_mode, None)
}

#[tauri::command]
fn end_session(state: tauri::State<'_, AppState>) -> Result<Option<Session>, String> {
    session::end_session_workflow(&state)
}

#[tauri::command]
fn add_typed_entry(state: tauri::State<'_, AppState>, text: String) -> Result<Entry, String> {
    session::add_typed_entry(&state, &text)
}

#[tauri::command]
fn add_untyped_entry(state: tauri::State<'_, AppState>, text: String) -> Result<Entry, String> {
    session::add_untyped_entry(&state, &text)
}

#[tauri::command]
fn start_record_mode(state: tauri::State<'_, AppState>) -> Result<Session, String> {
    let prefs = state.prefs.get();
    let chunk_duration_ms = prefs.record.chunk_seconds * 1000;

    let session = session::start_record_session(&state, &prefs)?;

    let mut guard = state.record_capture.lock();
    *guard = Some(RecordCapture::new(chunk_duration_ms));

    if let Some(ref record) = *guard {
        record
            .start(session.id.clone())
            .map_err(|e: audio::AudioError| e.to_string())?;
    }

    log::info!("Started record mode session: {}", session.id);
    Ok(session)
}

#[tauri::command]
fn stop_record_mode(state: tauri::State<'_, AppState>) -> Result<Option<Session>, String> {
    let record = {
        let mut guard = state.record_capture.lock();
        guard.take()
    };

    if let Some(record) = record {
        record
            .stop()
            .map_err(|e: audio::AudioError| e.to_string())?;
    }

    let session = session::end_record_session(&state)?;
    log::info!("Stopped record mode session");
    Ok(session)
}

#[tauri::command]
fn transcribe_record_chunk(state: tauri::State<'_, AppState>) -> Result<Option<Entry>, String> {
    let prefs = state.prefs.get();
    let record = state.record_capture.lock();

    if let Some(ref record) = *record {
        let needs_rotation =
            record.check_rotation_needed(prefs.record.max_hours, prefs.record.max_file_gb);

        if needs_rotation {
            log::info!("Record mode rotation triggered - max hours or file size reached");
            let old_session_id = record.get_session_id();

            let _ = record;

            if old_session_id.is_some() {
                session::end_record_session(&state).ok();
            }

            let chunk_duration_ms = prefs.record.chunk_seconds * 1000;
            let new_session =
                session::start_record_session(&state, &prefs).map_err(|e| e.to_string())?;

            *state.record_capture.lock() = Some(audio::RecordCapture::new(chunk_duration_ms));

            if let Some(ref mut record) = *state.record_capture.lock() {
                record
                    .start(new_session.id.clone())
                    .map_err(|e| e.to_string())?;
            }

            log::info!("Rotated to new session: {}", new_session.id);

            let record = state.record_capture.lock();
            if let Some(ref record) = *record {
                if let Some((session_id, audio_data, timestamp)) = record.get_and_clear_chunk() {
                    if audio_data.len() < 1600 {
                        return Ok(None);
                    }

                    match state.stt.transcribe(&audio_data, &prefs) {
                        Ok(result) => {
                            if !result.text.is_empty() {
                                if let Err(e) =
                                    audio::append_to_transcript_file(&session_id, &result.text)
                                {
                                    log::error!("Failed to write transcript: {}", e);
                                }

                                record.update_file_size(&session_id, result.text.len());

                                let entry = EntryCreate {
                                    id: uuid_v4(),
                                    session_id: session_id.clone(),
                                    started_at: timestamp as i64,
                                    ended_at: std::time::SystemTime::now()
                                        .duration_since(std::time::UNIX_EPOCH)
                                        .unwrap_or_default()
                                        .as_millis()
                                        as i64,
                                    text: result.text,
                                    source: db::SessionMode::Record,
                                    typed: false,
                                };

                                return state
                                    .db
                                    .create_entry(entry)
                                    .map(Some)
                                    .map_err(|e| e.to_string());
                            }
                        }
                        Err(e) => {
                            log::error!("Record chunk transcription failed: {}", e);
                        }
                    }
                }
            }
        } else {
            if let Some((session_id, audio_data, timestamp)) = record.get_and_clear_chunk() {
                if audio_data.len() < 1600 {
                    return Ok(None);
                }

                match state.stt.transcribe(&audio_data, &prefs) {
                    Ok(result) => {
                        if !result.text.is_empty() {
                            if let Err(e) =
                                audio::append_to_transcript_file(&session_id, &result.text)
                            {
                                log::error!("Failed to write transcript: {}", e);
                            }

                            record.update_file_size(&session_id, result.text.len());

                            let entry = EntryCreate {
                                id: uuid_v4(),
                                session_id: session_id.clone(),
                                started_at: timestamp as i64,
                                ended_at: std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .unwrap_or_default()
                                    .as_millis() as i64,
                                text: result.text,
                                source: db::SessionMode::Record,
                                typed: false,
                            };

                            return state
                                .db
                                .create_entry(entry)
                                .map(Some)
                                .map_err(|e| e.to_string());
                        }
                    }
                    Err(e) => {
                        log::error!("Record chunk transcription failed: {}", e);
                    }
                }
            }
        }
    }
    Ok(None)
}

#[tauri::command]
fn get_record_status(state: tauri::State<'_, AppState>) -> bool {
    let record = state.record_capture.lock();
    record.as_ref().map(|r| r.is_recording()).unwrap_or(false)
}

#[cfg(target_os = "macos")]
fn get_frontmost_app_name_internal() -> Option<String> {
    use objc2::rc::autoreleasepool;
    use objc2_app_kit::NSRunningApplication;

    autoreleasepool(|_| {
        let app = NSRunningApplication::currentApplication();
        let app_name = app.localizedName()?;
        Some(app_name.to_string())
    })
}

#[cfg(not(target_os = "macos"))]
fn get_frontmost_app_name_internal() -> Option<String> {
    None
}

fn uuid_v4() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let random: u64 = (timestamp as u64) ^ (std::process::id() as u64 * 0x517cc1b727220a95);
    format!("{:016x}-{:04x}", timestamp, random as u16)
}

fn silence_level_from_pref(level: &prefs::SilenceRms) -> SilenceLevel {
    match level {
        prefs::SilenceRms::Low => SilenceLevel::Low,
        prefs::SilenceRms::Medium => SilenceLevel::Medium,
        prefs::SilenceRms::High => SilenceLevel::High,
    }
}

fn session_mode_from_pref(mode: &prefs::ActivationMode) -> db::SessionMode {
    match mode {
        prefs::ActivationMode::Hold => db::SessionMode::Hold,
        prefs::ActivationMode::Toggle => db::SessionMode::Toggle,
    }
}

#[derive(Default)]
struct FinalizeGate {
    lock: std::sync::Mutex<()>,
}

impl FinalizeGate {
    fn run_for_session<R, Current, Finalize>(
        &self,
        expected_session_id: Option<&str>,
        current_session_id: Current,
        finalize: Finalize,
    ) -> Option<R>
    where
        Current: FnOnce() -> Option<String>,
        Finalize: FnOnce() -> R,
    {
        let _guard = self.lock.lock().unwrap();
        let active_session_id = current_session_id()?;

        if let Some(expected) = expected_session_id {
            if active_session_id != expected {
                return None;
            }
        }

        Some(finalize())
    }
}

fn stop_and_finalize_capture(
    audio: &AudioHandle,
    stt: &SttEngine,
    prefs: &Prefs,
    session_manager: &SessionManager,
    finalize_gate: &FinalizeGate,
    expected_session_id: Option<&str>,
) {
    let finalized = finalize_gate.run_for_session(
        expected_session_id,
        || session_manager.get_current_session_id(),
        || {
            if let Err(e) = audio.stop() {
                if !matches!(e, audio::AudioError::NotStarted) {
                    log::error!("Failed to stop audio capture: {}", e);
                }
            }

            let Some(mode) = session_manager.get_current_session_mode() else {
                log::warn!("No active session found when stopping capture");
                if let Err(e) = session_manager.end_session() {
                    log::error!("Failed to end session: {}", e);
                }
                return;
            };

            let prefs_snapshot = prefs.get();
            let audio_data = audio.get_buffer();
            if !audio_data.is_empty() {
                log::info!("Transcribing {} audio samples", audio_data.len());
                match stt.transcribe(&audio_data, &prefs_snapshot) {
                    Ok(result) => {
                        log::info!("Transcription result: {}", result.text);
                        if !result.text.is_empty() {
                            let typer_options = type_::TypeOptions {
                                method: type_::TypeMethod::Keystroke,
                                throttle_ms: prefs_snapshot.typing.throttle_ms as u64,
                                newline_append: prefs_snapshot.typing.newline_at_end,
                                clipboard_fallback: true,
                                detect_code_context: true,
                                detect_password_fields: true,
                            };
                            match type_::Typer::new(typer_options) {
                                Ok(typer) => {
                                    let typing_result = typer.type_text(&result.text);
                                    if let Err(e) = typing_result {
                                        log::error!("Failed to type text: {}", e);
                                        if let Err(e) =
                                            session_manager.add_entry(&result.text, false, mode)
                                        {
                                            log::error!("Failed to add untyped entry: {}", e);
                                        }
                                    } else if let Err(e) =
                                        session_manager.add_entry(&result.text, true, mode)
                                    {
                                        log::error!("Failed to add typed entry: {}", e);
                                    }
                                }
                                Err(e) => {
                                    log::error!("Failed to create typer: {}", e);
                                    if let Err(e) =
                                        session_manager.add_entry(&result.text, false, mode)
                                    {
                                        log::error!("Failed to add untyped entry: {}", e);
                                    }
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

            if let Err(e) = session_manager.end_session() {
                log::error!("Failed to end session: {}", e);
            }
        },
    );

    if finalized.is_none() {
        if expected_session_id.is_some() {
            log::debug!("Skipping finalize for stale or inactive toggle session");
        }
    }
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

    let session_manager = Arc::new(SessionManager::new(Arc::clone(&db)));

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
            let session_manager = Arc::clone(&session_manager);
            let finalize_gate = Arc::new(FinalizeGate::default());

            let initial_prefs = prefs.get();
            k.set_enabled(
                initial_prefs.hotkeys.left_chord,
                initial_prefs.hotkeys.right_chord,
            );
            audio.set_silence_threshold(silence_level_from_pref(&initial_prefs.silence_rms));

            k.on_activation(move |state, _source| match state {
                ActivationState::Active => {
                    let prefs_snapshot = prefs.get();
                    audio.set_silence_threshold(silence_level_from_pref(
                        &prefs_snapshot.silence_rms,
                    ));

                    match prefs_snapshot.mode {
                        prefs::ActivationMode::Hold => {
                            log::info!("Hotkey activated in hold mode - starting audio capture");

                            let mode = session_mode_from_pref(&prefs_snapshot.mode);
                            if let Err(e) =
                                session_manager.start_session(mode, &prefs_snapshot, None)
                            {
                                log::error!("Failed to create session: {}", e);
                                return;
                            }

                            if let Err(e) = audio.start() {
                                log::error!("Failed to start audio capture: {}", e);
                                if let Err(e) = session_manager.end_session() {
                                    log::error!("Failed to end session after start failure: {}", e);
                                }
                            }
                        }
                        prefs::ActivationMode::Toggle => {
                            if session_manager.is_active() || audio.is_recording() {
                                log::info!(
                                    "Toggle hotkey pressed while active - stopping audio capture"
                                );
                                let current_session_id = session_manager.get_current_session_id();
                                stop_and_finalize_capture(
                                    &audio,
                                    &stt,
                                    &prefs,
                                    &session_manager,
                                    &finalize_gate,
                                    current_session_id.as_deref(),
                                );
                                return;
                            }

                            log::info!("Toggle hotkey activated - starting audio capture");
                            let mode = session_mode_from_pref(&prefs_snapshot.mode);
                            if let Err(e) =
                                session_manager.start_session(mode, &prefs_snapshot, None)
                            {
                                log::error!("Failed to create session: {}", e);
                                return;
                            }

                            if let Err(e) = audio.start() {
                                log::error!("Failed to start audio capture: {}", e);
                                if let Err(e) = session_manager.end_session() {
                                    log::error!("Failed to end session after start failure: {}", e);
                                }
                                return;
                            }

                            let silence_timeout_ms =
                                (prefs_snapshot.silence_seconds * 1000.0).round() as u64;
                            let Some(toggle_session_id) = session_manager.get_current_session_id()
                            else {
                                log::warn!(
                                    "Toggle session missing after start; skipping silence watcher"
                                );
                                return;
                            };
                            let audio = Arc::clone(&audio);
                            let stt = Arc::clone(&stt);
                            let prefs = Arc::clone(&prefs);
                            let session_manager = Arc::clone(&session_manager);
                            let finalize_gate = Arc::clone(&finalize_gate);

                            std::thread::spawn(move || {
                                while audio.is_recording() {
                                    let Some(active_session_id) =
                                        session_manager.get_current_session_id()
                                    else {
                                        break;
                                    };

                                    if active_session_id != toggle_session_id {
                                        break;
                                    }

                                    if audio.get_silence_duration_ms() >= silence_timeout_ms {
                                        log::info!(
                                            "Silence timeout reached in toggle mode - stopping"
                                        );
                                        stop_and_finalize_capture(
                                            &audio,
                                            &stt,
                                            &prefs,
                                            &session_manager,
                                            &finalize_gate,
                                            Some(toggle_session_id.as_str()),
                                        );
                                        break;
                                    }

                                    std::thread::sleep(std::time::Duration::from_millis(100));
                                }
                            });
                        }
                    }
                }
                ActivationState::Inactive => {
                    let prefs_snapshot = prefs.get();
                    if matches!(prefs_snapshot.mode, prefs::ActivationMode::Hold) {
                        log::info!("Hotkey released in hold mode - stopping audio capture");
                        stop_and_finalize_capture(
                            &audio,
                            &stt,
                            &prefs,
                            &session_manager,
                            &finalize_gate,
                            None,
                        );
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
        session_manager,
        audio,
        record_capture: Arc::new(Mutex::new(None)),
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
            get_frontmost_app_name,
            start_session,
            end_session,
            add_typed_entry,
            add_untyped_entry,
            start_record_mode,
            stop_record_mode,
            transcribe_record_chunk,
            get_record_status,
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

#[cfg(test)]
mod tests {
    use super::FinalizeGate;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Barrier, Mutex};

    #[test]
    fn finalize_gate_allows_only_one_finalize_for_same_session() {
        let gate = Arc::new(FinalizeGate::default());
        let active_session_id = Arc::new(Mutex::new(Some("toggle-session".to_string())));
        let finalize_count = Arc::new(AtomicUsize::new(0));
        let barrier = Arc::new(Barrier::new(2));

        let mut handles = Vec::new();
        for _ in 0..2 {
            let gate = Arc::clone(&gate);
            let active_session_id = Arc::clone(&active_session_id);
            let finalize_count = Arc::clone(&finalize_count);
            let barrier = Arc::clone(&barrier);

            handles.push(std::thread::spawn(move || {
                barrier.wait();
                gate.run_for_session(
                    Some("toggle-session"),
                    || active_session_id.lock().unwrap().clone(),
                    || {
                        finalize_count.fetch_add(1, Ordering::SeqCst);
                        *active_session_id.lock().unwrap() = None;
                    },
                )
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(finalize_count.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn finalize_gate_rejects_stale_watcher_session() {
        let gate = FinalizeGate::default();
        let active_session_id = Arc::new(Mutex::new(Some("new-session".to_string())));
        let finalize_count = AtomicUsize::new(0);

        let finalized = gate.run_for_session(
            Some("old-session"),
            || active_session_id.lock().unwrap().clone(),
            || {
                finalize_count.fetch_add(1, Ordering::SeqCst);
                *active_session_id.lock().unwrap() = None;
            },
        );

        assert!(finalized.is_none());
        assert_eq!(finalize_count.load(Ordering::SeqCst), 0);
        assert_eq!(
            active_session_id.lock().unwrap().as_deref(),
            Some("new-session")
        );
    }
}
