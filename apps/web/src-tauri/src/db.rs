use anyhow::Result;
use directories::ProjectDirs;
use parking_lot::Mutex;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DbError {
    #[error("Failed to get app data directory")]
    NoAppDir,
    #[error("Database error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SessionMode {
    Hold,
    Toggle,
    Record,
}

impl Default for SessionMode {
    fn default() -> Self {
        Self::Hold
    }
}

impl std::fmt::Display for SessionMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SessionMode::Hold => write!(f, "hold"),
            SessionMode::Toggle => write!(f, "toggle"),
            SessionMode::Record => write!(f, "record"),
        }
    }
}

impl std::str::FromStr for SessionMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "hold" => Ok(SessionMode::Hold),
            "toggle" => Ok(SessionMode::Toggle),
            "record" => Ok(SessionMode::Record),
            _ => Err(format!("Unknown session mode: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub mode: SessionMode,
    pub started_at: i64,
    pub ended_at: Option<i64>,
    pub language: Option<String>,
    pub model_profile: String,
    pub translated: bool,
    pub app_name: Option<String>,
    pub chars_count: i64,
    pub words_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionCreate {
    pub id: String,
    pub mode: SessionMode,
    pub started_at: i64,
    pub language: Option<String>,
    pub model_profile: String,
    pub translated: bool,
    pub app_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    pub id: String,
    pub session_id: String,
    pub started_at: i64,
    pub ended_at: i64,
    pub text: String,
    pub source: SessionMode,
    pub typed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryCreate {
    pub id: String,
    pub session_id: String,
    pub started_at: i64,
    pub ended_at: i64,
    pub text: String,
    pub source: SessionMode,
    pub typed: bool,
}

pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    pub fn new() -> Result<Self, DbError> {
        let db_path = Self::get_db_path()?;
        let conn = Connection::open(&db_path)?;
        let db = Self {
            conn: Mutex::new(conn),
        };
        db.run_migrations()?;
        Ok(db)
    }

    #[cfg(test)]
    pub fn new_in_memory() -> Result<Self, DbError> {
        let conn = Connection::open_in_memory()?;
        let db = Self {
            conn: Mutex::new(conn),
        };
        db.run_migrations()?;
        Ok(db)
    }

    fn get_db_path() -> Result<PathBuf, DbError> {
        let proj_dirs = ProjectDirs::from("com", "stt", "sst").ok_or(DbError::NoAppDir)?;

        let data_dir = proj_dirs.data_dir().to_path_buf();
        std::fs::create_dir_all(&data_dir)?;

        Ok(data_dir.join("sst.db"))
    }

    fn run_migrations(&self) -> Result<(), DbError> {
        let conn = self.conn.lock();

        conn.execute(
            "CREATE TABLE IF NOT EXISTS sessions (
                id TEXT PRIMARY KEY,
                mode TEXT NOT NULL,
                started_at INTEGER NOT NULL,
                ended_at INTEGER,
                language TEXT,
                model_profile TEXT NOT NULL,
                translated INTEGER NOT NULL DEFAULT 0,
                app_name TEXT,
                chars_count INTEGER NOT NULL DEFAULT 0,
                words_count INTEGER NOT NULL DEFAULT 0
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS entries (
                id TEXT PRIMARY KEY,
                session_id TEXT NOT NULL REFERENCES sessions(id),
                started_at INTEGER NOT NULL,
                ended_at INTEGER NOT NULL,
                text TEXT NOT NULL,
                source TEXT NOT NULL,
                typed INTEGER NOT NULL DEFAULT 0
            )",
            [],
        )?;

        conn.execute(
            "CREATE VIRTUAL TABLE IF NOT EXISTS entry_search USING fts5(
                id,
                text,
                content='entries',
                content_rowid='rowid'
            )",
            [],
        )?;

        conn.execute(
            "CREATE TRIGGER IF NOT EXISTS entries_ai AFTER INSERT ON entries BEGIN
                INSERT INTO entry_search(id, text) VALUES (new.id, new.text);
            END",
            [],
        )?;

        conn.execute(
            "CREATE TRIGGER IF NOT EXISTS entries_ad AFTER DELETE ON entries BEGIN
                INSERT INTO entry_search(entry_search, id, text) VALUES('delete', old.id, old.text);
            END",
            [],
        )?;

        conn.execute(
            "CREATE TRIGGER IF NOT EXISTS entries_au AFTER UPDATE ON entries BEGIN
                INSERT INTO entry_search(entry_search, id, text) VALUES('delete', old.id, old.text);
                INSERT INTO entry_search(id, text) VALUES (new.id, new.text);
            END",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_sessions_started_at ON sessions(started_at)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_entries_session_id ON entries(session_id)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_entries_started_at ON entries(started_at)",
            [],
        )?;

        Ok(())
    }

    pub fn create_session(&self, session: SessionCreate) -> Result<Session, DbError> {
        let conn = self.conn.lock();

        conn.execute(
            "INSERT INTO sessions (id, mode, started_at, language, model_profile, translated, app_name)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                session.id,
                session.mode.to_string(),
                session.started_at,
                session.language,
                session.model_profile,
                session.translated as i32,
                session.app_name,
            ],
        )?;

        Ok(Session {
            id: session.id,
            mode: session.mode,
            started_at: session.started_at,
            ended_at: None,
            language: session.language,
            model_profile: session.model_profile,
            translated: session.translated,
            app_name: session.app_name,
            chars_count: 0,
            words_count: 0,
        })
    }

    pub fn get_session(&self, id: &str) -> Result<Option<Session>, DbError> {
        let conn = self.conn.lock();

        let mut stmt = conn.prepare(
            "SELECT id, mode, started_at, ended_at, language, model_profile, translated, app_name, chars_count, words_count
             FROM sessions WHERE id = ?1",
        )?;

        let mut rows = stmt.query(params![id])?;

        if let Some(row) = rows.next()? {
            let mode_str: String = row.get(1)?;
            Ok(Some(Session {
                id: row.get(0)?,
                mode: mode_str.parse().unwrap_or_default(),
                started_at: row.get(2)?,
                ended_at: row.get(3)?,
                language: row.get(4)?,
                model_profile: row.get(5)?,
                translated: row.get::<_, i32>(6)? != 0,
                app_name: row.get(7)?,
                chars_count: row.get(8)?,
                words_count: row.get(9)?,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn get_all_sessions(&self) -> Result<Vec<Session>, DbError> {
        let conn = self.conn.lock();

        let mut stmt = conn.prepare(
            "SELECT id, mode, started_at, ended_at, language, model_profile, translated, app_name, chars_count, words_count
             FROM sessions ORDER BY started_at DESC",
        )?;

        let rows = stmt.query_map([], |row| {
            let mode_str: String = row.get(1)?;
            Ok(Session {
                id: row.get(0)?,
                mode: mode_str.parse().unwrap_or_default(),
                started_at: row.get(2)?,
                ended_at: row.get(3)?,
                language: row.get(4)?,
                model_profile: row.get(5)?,
                translated: row.get::<_, i32>(6)? != 0,
                app_name: row.get(7)?,
                chars_count: row.get(8)?,
                words_count: row.get(9)?,
            })
        })?;

        let mut sessions = Vec::new();
        for session in rows {
            sessions.push(session?);
        }
        Ok(sessions)
    }

    pub fn update_session(
        &self,
        id: &str,
        ended_at: Option<i64>,
        chars_count: Option<i64>,
        words_count: Option<i64>,
    ) -> Result<Option<Session>, DbError> {
        let conn = self.conn.lock();

        if let Some(ended_at) = ended_at {
            conn.execute(
                "UPDATE sessions SET ended_at = ?1 WHERE id = ?2",
                params![ended_at, id],
            )?;
        }

        if let Some(chars_count) = chars_count {
            conn.execute(
                "UPDATE sessions SET chars_count = ?1 WHERE id = ?2",
                params![chars_count, id],
            )?;
        }

        if let Some(words_count) = words_count {
            conn.execute(
                "UPDATE sessions SET words_count = ?1 WHERE id = ?2",
                params![words_count, id],
            )?;
        }

        drop(conn);
        self.get_session(id)
    }

    pub fn delete_session(&self, id: &str) -> Result<bool, DbError> {
        let conn = self.conn.lock();

        conn.execute("DELETE FROM entries WHERE session_id = ?1", params![id])?;
        let rows_affected = conn.execute("DELETE FROM sessions WHERE id = ?1", params![id])?;

        Ok(rows_affected > 0)
    }

    pub fn create_entry(&self, entry: EntryCreate) -> Result<Entry, DbError> {
        let conn = self.conn.lock();

        conn.execute(
            "INSERT INTO entries (id, session_id, started_at, ended_at, text, source, typed)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                entry.id,
                entry.session_id,
                entry.started_at,
                entry.ended_at,
                entry.text,
                entry.source.to_string(),
                entry.typed as i32,
            ],
        )?;

        Ok(Entry {
            id: entry.id,
            session_id: entry.session_id,
            started_at: entry.started_at,
            ended_at: entry.ended_at,
            text: entry.text,
            source: entry.source,
            typed: entry.typed,
        })
    }

    pub fn get_entry(&self, id: &str) -> Result<Option<Entry>, DbError> {
        let conn = self.conn.lock();

        let mut stmt = conn.prepare(
            "SELECT id, session_id, started_at, ended_at, text, source, typed
             FROM entries WHERE id = ?1",
        )?;

        let mut rows = stmt.query(params![id])?;

        if let Some(row) = rows.next()? {
            let source_str: String = row.get(5)?;
            Ok(Some(Entry {
                id: row.get(0)?,
                session_id: row.get(1)?,
                started_at: row.get(2)?,
                ended_at: row.get(3)?,
                text: row.get(4)?,
                source: source_str.parse().unwrap_or_default(),
                typed: row.get::<_, i32>(6)? != 0,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn get_entries_by_session(&self, session_id: &str) -> Result<Vec<Entry>, DbError> {
        let conn = self.conn.lock();

        let mut stmt = conn.prepare(
            "SELECT id, session_id, started_at, ended_at, text, source, typed
             FROM entries WHERE session_id = ?1 ORDER BY started_at ASC",
        )?;

        let rows = stmt.query_map(params![session_id], |row| {
            let source_str: String = row.get(5)?;
            Ok(Entry {
                id: row.get(0)?,
                session_id: row.get(1)?,
                started_at: row.get(2)?,
                ended_at: row.get(3)?,
                text: row.get(4)?,
                source: source_str.parse().unwrap_or_default(),
                typed: row.get::<_, i32>(6)? != 0,
            })
        })?;

        let mut entries = Vec::new();
        for entry in rows {
            entries.push(entry?);
        }
        Ok(entries)
    }

    pub fn get_all_entries(&self) -> Result<Vec<Entry>, DbError> {
        let conn = self.conn.lock();

        let mut stmt = conn.prepare(
            "SELECT id, session_id, started_at, ended_at, text, source, typed
             FROM entries ORDER BY started_at DESC",
        )?;

        let rows = stmt.query_map([], |row| {
            let source_str: String = row.get(5)?;
            Ok(Entry {
                id: row.get(0)?,
                session_id: row.get(1)?,
                started_at: row.get(2)?,
                ended_at: row.get(3)?,
                text: row.get(4)?,
                source: source_str.parse().unwrap_or_default(),
                typed: row.get::<_, i32>(6)? != 0,
            })
        })?;

        let mut entries = Vec::new();
        for entry in rows {
            entries.push(entry?);
        }
        Ok(entries)
    }

    pub fn update_entry(
        &self,
        id: &str,
        text: Option<String>,
        typed: Option<bool>,
    ) -> Result<Option<Entry>, DbError> {
        let conn = self.conn.lock();

        if let Some(text) = text {
            conn.execute(
                "UPDATE entries SET text = ?1 WHERE id = ?2",
                params![text, id],
            )?;
        }

        if let Some(typed) = typed {
            conn.execute(
                "UPDATE entries SET typed = ?1 WHERE id = ?2",
                params![typed as i32, id],
            )?;
        }

        drop(conn);
        self.get_entry(id)
    }

    pub fn delete_entry(&self, id: &str) -> Result<bool, DbError> {
        let conn = self.conn.lock();

        let rows_affected = conn.execute("DELETE FROM entries WHERE id = ?1", params![id])?;

        Ok(rows_affected > 0)
    }

    pub fn search_entries(&self, query: &str) -> Result<Vec<Entry>, DbError> {
        let conn = self.conn.lock();

        let mut stmt = conn.prepare(
            "SELECT e.id, e.session_id, e.started_at, e.ended_at, e.text, e.source, e.typed
             FROM entries e
             JOIN entry_search es ON e.id = es.id
             WHERE entry_search MATCH ?1
             ORDER BY e.started_at DESC",
        )?;

        let rows = stmt.query_map(params![query], |row| {
            let source_str: String = row.get(5)?;
            Ok(Entry {
                id: row.get(0)?,
                session_id: row.get(1)?,
                started_at: row.get(2)?,
                ended_at: row.get(3)?,
                text: row.get(4)?,
                source: source_str.parse().unwrap_or_default(),
                typed: row.get::<_, i32>(6)? != 0,
            })
        })?;

        let mut entries = Vec::new();
        for entry in rows {
            entries.push(entry?);
        }
        Ok(entries)
    }
}

pub fn count_words(text: &str) -> i64 {
    text.split_whitespace().count() as i64
}

pub fn count_chars(text: &str) -> i64 {
    text.chars().count() as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_session() -> SessionCreate {
        SessionCreate {
            id: "test-session-1".to_string(),
            mode: SessionMode::Hold,
            started_at: 1000,
            language: Some("en".to_string()),
            model_profile: "base".to_string(),
            translated: false,
            app_name: Some("TestApp".to_string()),
        }
    }

    fn create_test_entry(session_id: &str) -> EntryCreate {
        EntryCreate {
            id: "test-entry-1".to_string(),
            session_id: session_id.to_string(),
            started_at: 1000,
            ended_at: 2000,
            text: "Hello world test".to_string(),
            source: SessionMode::Hold,
            typed: false,
        }
    }

    #[test]
    fn test_create_and_get_session() {
        let db = Database::new_in_memory().unwrap();
        let session_create = create_test_session();

        let created = db.create_session(session_create.clone()).unwrap();
        assert_eq!(created.id, session_create.id);
        assert_eq!(created.mode, session_create.mode);
        assert_eq!(created.started_at, session_create.started_at);
        assert_eq!(created.language, session_create.language);
        assert_eq!(created.model_profile, session_create.model_profile);
        assert_eq!(created.translated, session_create.translated);
        assert_eq!(created.app_name, session_create.app_name);
        assert_eq!(created.ended_at, None);
        assert_eq!(created.chars_count, 0);
        assert_eq!(created.words_count, 0);

        let retrieved = db.get_session(&session_create.id).unwrap();
        assert!(retrieved.is_some());
        let session = retrieved.unwrap();
        assert_eq!(session.id, session_create.id);
    }

    #[test]
    fn test_get_nonexistent_session() {
        let db = Database::new_in_memory().unwrap();
        let result = db.get_session("nonexistent").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_create_and_get_entry() {
        let db = Database::new_in_memory().unwrap();
        let session_id = "test-session-1".to_string();
        let session_create = SessionCreate {
            id: session_id.clone(),
            mode: SessionMode::Hold,
            started_at: 1000,
            language: Some("en".to_string()),
            model_profile: "base".to_string(),
            translated: false,
            app_name: None,
        };
        db.create_session(session_create).unwrap();

        let entry_create = EntryCreate {
            id: "test-entry-1".to_string(),
            session_id: session_id.clone(),
            started_at: 1000,
            ended_at: 2000,
            text: "Hello world test".to_string(),
            source: SessionMode::Hold,
            typed: false,
        };
        let created = db.create_entry(entry_create.clone()).unwrap();
        assert_eq!(created.id, entry_create.id);
        assert_eq!(created.session_id, entry_create.session_id);
        assert_eq!(created.text, entry_create.text);
        assert_eq!(created.source, entry_create.source);
        assert_eq!(created.typed, entry_create.typed);

        let retrieved = db.get_entry(&entry_create.id).unwrap();
        assert!(retrieved.is_some());
        let entry = retrieved.unwrap();
        assert_eq!(entry.id, entry_create.id);
    }

    #[test]
    fn test_get_entries_by_session() {
        let db = Database::new_in_memory().unwrap();
        let session_id = "test-session-1".to_string();
        let session_create = SessionCreate {
            id: session_id.clone(),
            mode: SessionMode::Hold,
            started_at: 1000,
            language: Some("en".to_string()),
            model_profile: "base".to_string(),
            translated: false,
            app_name: None,
        };
        db.create_session(session_create).unwrap();

        let entry1 = EntryCreate {
            id: "entry-1".to_string(),
            session_id: session_id.clone(),
            started_at: 1000,
            ended_at: 2000,
            text: "First entry".to_string(),
            source: SessionMode::Hold,
            typed: false,
        };
        let entry2 = EntryCreate {
            id: "entry-2".to_string(),
            session_id: session_id.clone(),
            started_at: 2000,
            ended_at: 3000,
            text: "Second entry".to_string(),
            source: SessionMode::Hold,
            typed: true,
        };
        db.create_entry(entry1).unwrap();
        db.create_entry(entry2).unwrap();

        let entries = db.get_entries_by_session(&session_id).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].text, "First entry");
        assert_eq!(entries[1].text, "Second entry");
    }

    #[test]
    fn test_get_all_sessions() {
        let db = Database::new_in_memory().unwrap();

        let session1 = SessionCreate {
            id: "session-1".to_string(),
            mode: SessionMode::Hold,
            started_at: 1000,
            language: Some("en".to_string()),
            model_profile: "base".to_string(),
            translated: false,
            app_name: None,
        };
        let session2 = SessionCreate {
            id: "session-2".to_string(),
            mode: SessionMode::Toggle,
            started_at: 2000,
            language: Some("es".to_string()),
            model_profile: "small".to_string(),
            translated: true,
            app_name: None,
        };

        db.create_session(session1).unwrap();
        db.create_session(session2).unwrap();

        let sessions = db.get_all_sessions().unwrap();
        assert_eq!(sessions.len(), 2);
    }

    #[test]
    fn test_update_session() {
        let db = Database::new_in_memory().unwrap();
        let session_create = create_test_session();
        db.create_session(session_create.clone()).unwrap();

        let updated = db
            .update_session(&session_create.id, Some(5000), Some(100), Some(20))
            .unwrap()
            .unwrap();

        assert_eq!(updated.ended_at, Some(5000));
        assert_eq!(updated.chars_count, 100);
        assert_eq!(updated.words_count, 20);
    }

    #[test]
    fn test_delete_session() {
        let db = Database::new_in_memory().unwrap();
        let session_id = "test-session-1".to_string();
        let session_create = SessionCreate {
            id: session_id.clone(),
            mode: SessionMode::Hold,
            started_at: 1000,
            language: Some("en".to_string()),
            model_profile: "base".to_string(),
            translated: false,
            app_name: None,
        };
        db.create_session(session_create).unwrap();

        let entry_create = EntryCreate {
            id: "test-entry-1".to_string(),
            session_id: session_id.clone(),
            started_at: 1000,
            ended_at: 2000,
            text: "Test entry".to_string(),
            source: SessionMode::Hold,
            typed: false,
        };
        db.create_entry(entry_create).unwrap();

        let deleted = db.delete_session(&session_id).unwrap();
        assert!(deleted);

        let session = db.get_session(&session_id).unwrap();
        assert!(session.is_none());

        let entries = db.get_entries_by_session(&session_id).unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn test_update_entry() {
        let db = Database::new_in_memory().unwrap();
        let session_id = "test-session-1".to_string();
        let session_create = SessionCreate {
            id: session_id.clone(),
            mode: SessionMode::Hold,
            started_at: 1000,
            language: Some("en".to_string()),
            model_profile: "base".to_string(),
            translated: false,
            app_name: None,
        };
        db.create_session(session_create).unwrap();

        let entry_id = "test-entry-1".to_string();
        let entry_create = EntryCreate {
            id: entry_id.clone(),
            session_id: session_id.clone(),
            started_at: 1000,
            ended_at: 2000,
            text: "Original text".to_string(),
            source: SessionMode::Hold,
            typed: false,
        };
        db.create_entry(entry_create).unwrap();

        let updated = db
            .update_entry(&entry_id, Some("Updated text".to_string()), Some(true))
            .unwrap()
            .unwrap();

        assert_eq!(updated.text, "Updated text");
        assert!(updated.typed);
    }

    #[test]
    fn test_delete_entry() {
        let db = Database::new_in_memory().unwrap();
        let session_id = "test-session-1".to_string();
        let session_create = SessionCreate {
            id: session_id.clone(),
            mode: SessionMode::Hold,
            started_at: 1000,
            language: Some("en".to_string()),
            model_profile: "base".to_string(),
            translated: false,
            app_name: None,
        };
        db.create_session(session_create).unwrap();

        let entry_id = "test-entry-1".to_string();
        let entry_create = EntryCreate {
            id: entry_id.clone(),
            session_id: session_id.clone(),
            started_at: 1000,
            ended_at: 2000,
            text: "Test entry".to_string(),
            source: SessionMode::Hold,
            typed: false,
        };
        db.create_entry(entry_create).unwrap();

        let deleted = db.delete_entry(&entry_id).unwrap();
        assert!(deleted);

        let entry = db.get_entry(&entry_id).unwrap();
        assert!(entry.is_none());
    }

    #[test]
    fn test_fts5_search() {
        let db = Database::new_in_memory().unwrap();
        let session_id = "test-session-1".to_string();
        let session_create = SessionCreate {
            id: session_id.clone(),
            mode: SessionMode::Hold,
            started_at: 1000,
            language: Some("en".to_string()),
            model_profile: "base".to_string(),
            translated: false,
            app_name: None,
        };
        db.create_session(session_create).unwrap();

        let entry1 = EntryCreate {
            id: "entry-1".to_string(),
            session_id: session_id.clone(),
            started_at: 1000,
            ended_at: 2000,
            text: "Hello world this is a test".to_string(),
            source: SessionMode::Hold,
            typed: false,
        };
        let entry2 = EntryCreate {
            id: "entry-2".to_string(),
            session_id: session_id.clone(),
            started_at: 2000,
            ended_at: 3000,
            text: "Another entry with different content".to_string(),
            source: SessionMode::Hold,
            typed: false,
        };

        db.create_entry(entry1).unwrap();
        db.create_entry(entry2).unwrap();

        let results = db.search_entries("hello").unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].text.contains("Hello"));

        let results2 = db.search_entries("world").unwrap();
        assert_eq!(results2.len(), 1);

        let results3 = db.search_entries("nonexistent").unwrap();
        assert!(results3.is_empty());
    }

    #[test]
    fn test_fts5_search_multiple_words() {
        let db = Database::new_in_memory().unwrap();
        let session_id = "test-session-1".to_string();
        let session_create = SessionCreate {
            id: session_id.clone(),
            mode: SessionMode::Hold,
            started_at: 1000,
            language: Some("en".to_string()),
            model_profile: "base".to_string(),
            translated: false,
            app_name: None,
        };
        db.create_session(session_create).unwrap();

        let entry = EntryCreate {
            id: "entry-1".to_string(),
            session_id: session_id.clone(),
            started_at: 1000,
            ended_at: 2000,
            text: "The quick brown fox jumps".to_string(),
            source: SessionMode::Hold,
            typed: false,
        };
        db.create_entry(entry).unwrap();

        let results = db.search_entries("quick brown").unwrap();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_fts5_search_after_update() {
        let db = Database::new_in_memory().unwrap();
        let session_id = "test-session-1".to_string();
        let session_create = SessionCreate {
            id: session_id.clone(),
            mode: SessionMode::Hold,
            started_at: 1000,
            language: Some("en".to_string()),
            model_profile: "base".to_string(),
            translated: false,
            app_name: None,
        };
        db.create_session(session_create).unwrap();

        let entry_id = "entry-1".to_string();
        let entry_create = EntryCreate {
            id: entry_id.clone(),
            session_id: session_id.clone(),
            started_at: 1000,
            ended_at: 2000,
            text: "First entry text".to_string(),
            source: SessionMode::Hold,
            typed: false,
        };
        db.create_entry(entry_create).unwrap();

        let entry_id2 = "entry-2".to_string();
        let entry_create2 = EntryCreate {
            id: entry_id2.clone(),
            session_id: session_id.clone(),
            started_at: 2000,
            ended_at: 3000,
            text: "Second entry text".to_string(),
            source: SessionMode::Hold,
            typed: false,
        };
        db.create_entry(entry_create2).unwrap();

        let results = db.search_entries("second").unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].text.contains("Second"));
    }

    #[test]
    fn test_fts5_search_after_delete() {
        let db = Database::new_in_memory().unwrap();
        let session_id = "test-session-1".to_string();
        let session_create = SessionCreate {
            id: session_id.clone(),
            mode: SessionMode::Hold,
            started_at: 1000,
            language: Some("en".to_string()),
            model_profile: "base".to_string(),
            translated: false,
            app_name: None,
        };
        db.create_session(session_create).unwrap();

        let entry_id = "entry-1".to_string();
        let entry_create = EntryCreate {
            id: entry_id.clone(),
            session_id: session_id.clone(),
            started_at: 1000,
            ended_at: 2000,
            text: "Searchable content".to_string(),
            source: SessionMode::Hold,
            typed: false,
        };
        db.create_entry(entry_create).unwrap();

        let results_before = db.search_entries("searchable").unwrap();
        assert_eq!(results_before.len(), 1);

        let deleted = db.delete_entry(&entry_id).unwrap();
        assert!(deleted);
    }

    #[test]
    fn test_timestamps() {
        let db = Database::new_in_memory().unwrap();
        let session_id = "session-timestamp".to_string();
        let session_create = SessionCreate {
            id: session_id.clone(),
            mode: SessionMode::Hold,
            started_at: 1609459200000,
            language: None,
            model_profile: "base".to_string(),
            translated: false,
            app_name: None,
        };
        db.create_session(session_create).unwrap();

        let session = db.get_session(&session_id).unwrap().unwrap();
        assert_eq!(session.started_at, 1609459200000);

        let entry_create = EntryCreate {
            id: "entry-timestamp".to_string(),
            session_id: session_id.clone(),
            started_at: 1609459200000,
            ended_at: 1609459260000,
            text: "Test".to_string(),
            source: SessionMode::Hold,
            typed: false,
        };
        db.create_entry(entry_create).unwrap();

        let entry = db.get_entry("entry-timestamp").unwrap().unwrap();
        assert_eq!(entry.started_at, 1609459200000);
        assert_eq!(entry.ended_at, 1609459260000);
    }

    #[test]
    fn test_word_char_counts() {
        let db = Database::new_in_memory().unwrap();
        let session_id = "test-session-1".to_string();
        let session_create = SessionCreate {
            id: session_id.clone(),
            mode: SessionMode::Hold,
            started_at: 1000,
            language: Some("en".to_string()),
            model_profile: "base".to_string(),
            translated: false,
            app_name: None,
        };
        db.create_session(session_create).unwrap();

        let text = "Hello world this is a test message";
        let words = count_words(text);
        let chars = count_chars(text);

        assert_eq!(words, 7);
        assert_eq!(chars, text.len() as i64);

        let entry = EntryCreate {
            id: "entry-counts".to_string(),
            session_id: session_id.clone(),
            started_at: 1000,
            ended_at: 2000,
            text: text.to_string(),
            source: SessionMode::Hold,
            typed: false,
        };
        db.create_entry(entry).unwrap();

        db.update_session(&session_id, None, Some(chars), Some(words))
            .unwrap()
            .unwrap();

        let session = db.get_session(&session_id).unwrap().unwrap();
        assert_eq!(session.chars_count, chars);
        assert_eq!(session.words_count, words);
    }

    #[test]
    fn test_migration_creates_tables() {
        let db = Database::new_in_memory().unwrap();

        let session_create = create_test_session();
        db.create_session(session_create.clone()).unwrap();

        let entry_create = create_test_entry(&session_create.id);
        db.create_entry(entry_create).unwrap();

        let sessions = db.get_all_sessions().unwrap();
        assert_eq!(sessions.len(), 1);

        let entries = db.get_all_entries().unwrap();
        assert_eq!(entries.len(), 1);
    }

    #[test]
    fn test_migration_creates_indexes() {
        let db = Database::new_in_memory().unwrap();

        let conn = db.conn.lock();
        let mut stmt = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='index' AND name='idx_sessions_started_at'")
            .unwrap();
        let result: Vec<String> = stmt
            .query_map([], |row| row.get(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();
        assert_eq!(result.len(), 1);

        let mut stmt = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='index' AND name='idx_entries_session_id'")
            .unwrap();
        let result: Vec<String> = stmt
            .query_map([], |row| row.get(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();
        assert_eq!(result.len(), 1);

        let mut stmt = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='index' AND name='idx_entries_started_at'")
            .unwrap();
        let result: Vec<String> = stmt
            .query_map([], |row| row.get(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_migration_creates_fts5_table() {
        let db = Database::new_in_memory().unwrap();

        let conn = db.conn.lock();
        let mut stmt = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='entry_search'")
            .unwrap();
        let result: Vec<String> = stmt
            .query_map([], |row| row.get(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();
        assert_eq!(result.len(), 1);

        let mut stmt = conn
            .prepare(
                "SELECT name FROM sqlite_master WHERE type='trigger' AND name LIKE 'entries_%'",
            )
            .unwrap();
        let triggers: Vec<String> = stmt
            .query_map([], |row| row.get(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect();
        assert!(triggers.len() >= 3);
    }

    #[test]
    fn test_session_mode_serialization() {
        let session = SessionCreate {
            id: "mode-test".to_string(),
            mode: SessionMode::Toggle,
            started_at: 1000,
            language: None,
            model_profile: "base".to_string(),
            translated: false,
            app_name: None,
        };

        let db = Database::new_in_memory().unwrap();
        let created = db.create_session(session).unwrap();
        assert_eq!(created.mode, SessionMode::Toggle);

        let retrieved = db.get_session("mode-test").unwrap().unwrap();
        assert_eq!(retrieved.mode, SessionMode::Toggle);
    }

    #[test]
    fn test_typed_field() {
        let db = Database::new_in_memory().unwrap();
        let session_create = create_test_session();
        db.create_session(session_create.clone()).unwrap();

        let entry = EntryCreate {
            id: "typed-entry".to_string(),
            session_id: session_create.id,
            started_at: 1000,
            ended_at: 2000,
            text: "Typed content".to_string(),
            source: SessionMode::Hold,
            typed: true,
        };
        db.create_entry(entry.clone()).unwrap();

        let retrieved = db.get_entry(&entry.id).unwrap().unwrap();
        assert!(retrieved.typed);
    }

    #[test]
    fn test_app_name_field() {
        let session = SessionCreate {
            id: "app-test".to_string(),
            mode: SessionMode::Hold,
            started_at: 1000,
            language: None,
            model_profile: "base".to_string(),
            translated: false,
            app_name: Some("Safari".to_string()),
        };

        let db = Database::new_in_memory().unwrap();
        let created = db.create_session(session).unwrap();
        assert_eq!(created.app_name, Some("Safari".to_string()));

        let retrieved = db.get_session("app-test").unwrap().unwrap();
        assert_eq!(retrieved.app_name, Some("Safari".to_string()));
    }

    #[test]
    fn test_multiple_sessions_with_entries() {
        let db = Database::new_in_memory().unwrap();

        let session1_id = "session-1".to_string();
        let session2_id = "session-2".to_string();

        let session1 = SessionCreate {
            id: session1_id.clone(),
            mode: SessionMode::Hold,
            started_at: 1000,
            language: None,
            model_profile: "base".to_string(),
            translated: false,
            app_name: None,
        };
        let session2 = SessionCreate {
            id: session2_id.clone(),
            mode: SessionMode::Toggle,
            started_at: 2000,
            language: None,
            model_profile: "base".to_string(),
            translated: false,
            app_name: None,
        };

        db.create_session(session1).unwrap();
        db.create_session(session2).unwrap();

        db.create_entry(EntryCreate {
            id: "e1".to_string(),
            session_id: session1_id.clone(),
            started_at: 1000,
            ended_at: 2000,
            text: "Session 1 entry".to_string(),
            source: SessionMode::Hold,
            typed: false,
        })
        .unwrap();

        db.create_entry(EntryCreate {
            id: "e2".to_string(),
            session_id: session2_id.clone(),
            started_at: 2000,
            ended_at: 3000,
            text: "Session 2 entry".to_string(),
            source: SessionMode::Toggle,
            typed: false,
        })
        .unwrap();

        let entries1 = db.get_entries_by_session(&session1_id).unwrap();
        let entries2 = db.get_entries_by_session(&session2_id).unwrap();

        assert_eq!(entries1.len(), 1);
        assert_eq!(entries2.len(), 1);
        assert!(entries1[0].text.contains("Session 1"));
        assert!(entries2[0].text.contains("Session 2"));
    }

    #[test]
    fn test_translated_field() {
        let session = SessionCreate {
            id: "translated-test".to_string(),
            mode: SessionMode::Hold,
            started_at: 1000,
            language: Some("es".to_string()),
            model_profile: "base".to_string(),
            translated: true,
            app_name: None,
        };

        let db = Database::new_in_memory().unwrap();
        let created = db.create_session(session).unwrap();
        assert!(created.translated);

        let retrieved = db.get_session("translated-test").unwrap().unwrap();
        assert!(retrieved.translated);
    }

    #[test]
    fn test_idempotent_migrations() {
        let db = Database::new_in_memory().unwrap();
        db.run_migrations().unwrap();

        let session_create = create_test_session();
        db.create_session(session_create).unwrap();

        let sessions = db.get_all_sessions().unwrap();
        assert_eq!(sessions.len(), 1);
    }
}
