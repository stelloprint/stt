use crate::db::{count_chars, count_words, Database, Entry, EntryCreate, Session, SessionCreate};
use crate::prefs::Preferences;
use crate::AppState;
use parking_lot::RwLock;
use std::sync::Arc;

pub struct SessionManager {
    db: Arc<Database>,
    current_session: RwLock<Option<CurrentSession>>,
}

struct CurrentSession {
    id: String,
    started_at: i64,
    mode: crate::db::SessionMode,
}

fn model_profile_to_string(profile: &crate::prefs::ModelProfile) -> String {
    match profile {
        crate::prefs::ModelProfile::EnglishSmall => "small.en".to_string(),
        crate::prefs::ModelProfile::MultilingualSmall => "multilingual-small".to_string(),
        crate::prefs::ModelProfile::MultilingualMedium => "multilingual-medium".to_string(),
    }
}

impl SessionManager {
    pub fn new(db: Arc<Database>) -> Self {
        Self {
            db,
            current_session: RwLock::new(None),
        }
    }

    pub fn start_session(
        &self,
        mode: crate::db::SessionMode,
        prefs: &Preferences,
        app_name: Option<String>,
    ) -> Result<Session, String> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_millis() as i64;

        let session = SessionCreate {
            id: uuid_v4(),
            mode,
            started_at: now,
            language: None,
            model_profile: model_profile_to_string(&prefs.model_profile),
            translated: prefs.translate_to_english,
            app_name,
        };

        let created = self.db.create_session(session).map_err(|e| e.to_string())?;

        *self.current_session.write() = Some(CurrentSession {
            id: created.id.clone(),
            started_at: now,
            mode,
        });

        log::info!("Started session: {}", created.id);
        Ok(created)
    }

    pub fn end_session(&self) -> Result<Option<Session>, String> {
        let current = self.current_session.write().take();
        if let Some(session) = current {
            let entries = self
                .db
                .get_entries_by_session(&session.id)
                .map_err(|e| e.to_string())?;

            let total_chars: i64 = entries.iter().map(|e| count_chars(&e.text)).sum();
            let total_words: i64 = entries.iter().map(|e| count_words(&e.text)).sum();

            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|e| e.to_string())?
                .as_millis() as i64;

            let updated = self
                .db
                .update_session(&session.id, Some(now), Some(total_chars), Some(total_words))
                .map_err(|e| e.to_string())?;

            log::info!(
                "Ended session: {} with {} chars, {} words",
                session.id,
                total_chars,
                total_words
            );
            Ok(updated)
        } else {
            Ok(None)
        }
    }

    pub fn add_entry(
        &self,
        text: &str,
        typed: bool,
        source: crate::db::SessionMode,
    ) -> Result<Entry, String> {
        let current = self.current_session.read();
        if let Some(session) = current.as_ref() {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|e| e.to_string())?
                .as_millis() as i64;

            let entry = EntryCreate {
                id: uuid_v4(),
                session_id: session.id.clone(),
                started_at: session.started_at,
                ended_at: now,
                text: text.to_string(),
                source,
                typed,
            };

            self.db.create_entry(entry).map_err(|e| e.to_string())
        } else {
            Err("No active session".to_string())
        }
    }

    pub fn get_current_session_id(&self) -> Option<String> {
        self.current_session.read().as_ref().map(|s| s.id.clone())
    }

    pub fn is_active(&self) -> bool {
        self.current_session.read().is_some()
    }

    pub fn get_current_session_mode(&self) -> Option<crate::db::SessionMode> {
        self.current_session.read().as_ref().map(|s| s.mode)
    }
}

fn uuid_v4() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let timestamp_lower = (timestamp & 0xFFFFFFFFFFFFFFFF) as u64;
    let pid = std::process::id() as u64;
    let random: u64 = timestamp_lower.wrapping_mul(pid.wrapping_add(1));
    format!("{:016x}-{:04x}", timestamp_lower, random as u16)
}

pub fn create_session_workflow(
    state: &AppState,
    mode: crate::db::SessionMode,
    _source: Option<crate::keys::ActivationSource>,
) -> Result<Session, String> {
    let prefs = state.prefs.get();
    let app_name = get_frontmost_app_name_internal();
    state.session_manager.start_session(mode, &prefs, app_name)
}

pub fn end_session_workflow(state: &AppState) -> Result<Option<Session>, String> {
    state.session_manager.end_session()
}

pub fn add_typed_entry(state: &AppState, text: &str) -> Result<Entry, String> {
    let mode = state
        .session_manager
        .get_current_session_mode()
        .ok_or("No active session")?;
    state.session_manager.add_entry(text, true, mode)
}

pub fn add_untyped_entry(state: &AppState, text: &str) -> Result<Entry, String> {
    let mode = state
        .session_manager
        .get_current_session_mode()
        .ok_or("No active session")?;
    state.session_manager.add_entry(text, false, mode)
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

pub fn start_record_session(
    state: &crate::AppState,
    prefs: &crate::prefs::Preferences,
) -> Result<crate::db::Session, String> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_millis() as i64;

    let session = crate::db::SessionCreate {
        id: uuid_v4(),
        mode: crate::db::SessionMode::Record,
        started_at: now,
        language: None,
        model_profile: model_profile_to_string(&prefs.model_profile),
        translated: prefs.translate_to_english,
        app_name: None,
    };

    state.db.create_session(session).map_err(|e| e.to_string())
}

pub fn end_record_session(state: &crate::AppState) -> Result<Option<crate::db::Session>, String> {
    let sessions = state.db.get_all_sessions().map_err(|e| e.to_string())?;

    if let Some(session) = sessions
        .into_iter()
        .find(|s| s.mode == crate::db::SessionMode::Record && s.ended_at.is_none())
    {
        let entries = state
            .db
            .get_entries_by_session(&session.id)
            .map_err(|e| e.to_string())?;

        let total_chars: i64 = entries.iter().map(|e| count_chars(&e.text)).sum();
        let total_words: i64 = entries.iter().map(|e| count_words(&e.text)).sum();

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_millis() as i64;

        let updated = state
            .db
            .update_session(&session.id, Some(now), Some(total_chars), Some(total_words))
            .map_err(|e| e.to_string())?;

        log::info!(
            "Ended record session: {} with {} chars, {} words",
            session.id,
            total_chars,
            total_words
        );
        Ok(updated)
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;

    fn test_prefs() -> Preferences {
        Preferences::default()
    }

    #[test]
    fn test_hold_mode_session_persists_hold_source_entry() {
        let db = Database::new_in_memory().unwrap();
        let manager = SessionManager::new(Arc::new(db));

        let prefs = test_prefs();
        manager
            .start_session(crate::db::SessionMode::Hold, &prefs, None)
            .unwrap();

        manager
            .add_entry("hello world", true, crate::db::SessionMode::Hold)
            .unwrap();

        let ended = manager.end_session().unwrap().unwrap();
        assert_eq!(ended.mode, crate::db::SessionMode::Hold);
        assert!(ended.chars_count > 0);
        assert!(ended.words_count > 0);
    }

    #[test]
    fn test_toggle_mode_session_persists_toggle_source_entry() {
        let db = Database::new_in_memory().unwrap();
        let manager = SessionManager::new(Arc::new(db));

        let prefs = test_prefs();
        manager
            .start_session(crate::db::SessionMode::Toggle, &prefs, None)
            .unwrap();

        manager
            .add_entry("testing toggle", true, crate::db::SessionMode::Toggle)
            .unwrap();

        let ended = manager.end_session().unwrap().unwrap();
        assert_eq!(ended.mode, crate::db::SessionMode::Toggle);
    }

    #[test]
    fn test_untyped_entry_persists_correctly() {
        let db = Database::new_in_memory().unwrap();
        let manager = SessionManager::new(Arc::new(db));

        let prefs = test_prefs();
        manager
            .start_session(crate::db::SessionMode::Hold, &prefs, None)
            .unwrap();

        manager
            .add_entry("untyped text", false, crate::db::SessionMode::Hold)
            .unwrap();

        let ended = manager.end_session().unwrap().unwrap();
        assert_eq!(ended.chars_count, 12);
        assert_eq!(ended.words_count, 2);
    }

    #[test]
    fn test_session_totals_computed_from_entries() {
        let db = Database::new_in_memory().unwrap();
        let manager = SessionManager::new(Arc::new(db));

        let prefs = test_prefs();
        manager
            .start_session(crate::db::SessionMode::Hold, &prefs, None)
            .unwrap();

        manager
            .add_entry("hello", true, crate::db::SessionMode::Hold)
            .unwrap();
        manager
            .add_entry("world", true, crate::db::SessionMode::Hold)
            .unwrap();

        let ended = manager.end_session().unwrap().unwrap();
        assert_eq!(ended.chars_count, 10);
        assert_eq!(ended.words_count, 2);
    }

    #[test]
    fn test_mixed_typed_untyped_entries() {
        let db = Database::new_in_memory().unwrap();
        let manager = SessionManager::new(Arc::new(db));

        let prefs = test_prefs();
        manager
            .start_session(crate::db::SessionMode::Hold, &prefs, None)
            .unwrap();

        manager
            .add_entry("typed content", true, crate::db::SessionMode::Hold)
            .unwrap();
        manager
            .add_entry("untyped content", false, crate::db::SessionMode::Hold)
            .unwrap();

        let ended = manager.end_session().unwrap().unwrap();
        assert_eq!(ended.chars_count, 28);
        assert_eq!(ended.words_count, 4);
    }

    #[test]
    fn test_end_session_without_active_session_returns_none() {
        let db = Database::new_in_memory().unwrap();
        let manager = SessionManager::new(Arc::new(db));

        let result = manager.end_session().unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_add_entry_without_active_session_returns_error() {
        let db = Database::new_in_memory().unwrap();
        let manager = SessionManager::new(Arc::new(db));

        let result = manager.add_entry("test", true, crate::db::SessionMode::Hold);
        assert!(result.is_err());
    }
}
