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
