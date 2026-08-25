//! Session store — in-memory + SQLite-backed implementations.
//!
//! Ported from `core/src/session/store.ts`.
//! The TS version uses Drizzle ORM with SQLite; here we provide
//! an in-memory store and a trait-compatible SQLite store using `rusqlite`.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use async_trait::async_trait;
use tokio::sync::RwLock;

use crate::schema::ids::SessionID;
use crate::schema::session::{SessionInfo, SessionMessage};

use super::SessionStore;

struct SessionData {
    info: SessionInfo,
    messages: Vec<SessionMessage>,
}

pub struct InMemorySessionStore {
    sessions: Arc<RwLock<HashMap<String, SessionData>>>,
}

impl InMemorySessionStore {
    pub fn new() -> Self {
        Self { sessions: Arc::new(RwLock::new(HashMap::new())) }
    }
}

impl Default for InMemorySessionStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SessionStore for InMemorySessionStore {
    async fn get(&self, id: &SessionID) -> Option<SessionInfo> {
        self.sessions.read().await.get(id.as_str()).map(|d| d.info.clone())
    }

    async fn list(&self, limit: usize, offset: usize) -> Vec<SessionInfo> {
        let sessions = self.sessions.read().await;
        let mut all: Vec<SessionInfo> = sessions.values().map(|d| d.info.clone()).collect();
        all.sort_by(|a, b| b.time.updated.cmp(&a.time.updated));
        all.into_iter().skip(offset).take(limit).collect()
    }

    async fn create(&self, info: SessionInfo) -> SessionInfo {
        let id = info.id.0.clone();
        self.sessions.write().await.insert(id, SessionData { info: info.clone(), messages: vec![] });
        info
    }

    async fn update(&self, info: SessionInfo) -> SessionInfo {
        let id = info.id.0.clone();
        let mut sessions = self.sessions.write().await;
        if let Some(data) = sessions.get_mut(&id) {
            data.info = info.clone();
        }
        info
    }

    async fn delete(&self, id: &SessionID) -> bool {
        self.sessions.write().await.remove(id.as_str()).is_some()
    }

    async fn context(&self, id: &SessionID) -> Option<Vec<SessionMessage>> {
        self.sessions.read().await.get(id.as_str()).map(|d| d.messages.clone())
    }

    async fn append_message(&self, session_id: &SessionID, message: SessionMessage) -> bool {
        let mut sessions = self.sessions.write().await;
        if let Some(data) = sessions.get_mut(session_id.as_str()) {
            data.messages.push(message);
            true
        } else {
            false
        }
    }

    async fn get_message(&self, session_id: &SessionID, message_id: &str) -> Option<SessionMessage> {
        let sessions = self.sessions.read().await;
        sessions.get(session_id.as_str()).and_then(|d| {
            d.messages.iter().find(|m| {
                let mid = match m {
                    SessionMessage::User { id, .. } => id.0.as_str(),
                    SessionMessage::Assistant { id, .. } => id.0.as_str(),
                    SessionMessage::System { id, .. } => id.0.as_str(),
                    SessionMessage::Shell { id, .. } => id.0.as_str(),
                    SessionMessage::Synthetic { id, .. } => id.0.as_str(),
                    SessionMessage::AgentSwitched { id, .. } => id.0.as_str(),
                    SessionMessage::ModelSwitched { id, .. } => id.0.as_str(),
                    SessionMessage::Compaction { id, .. } => id.0.as_str(),
                };
                mid == message_id
            }).cloned()
        })
    }
}

/// SQLite-backed session store.
///
/// Schema (matching the TS Drizzle tables):
/// ```sql
/// CREATE TABLE IF NOT EXISTS session (
///     id TEXT PRIMARY KEY,
///     data TEXT NOT NULL,
///     time_created INTEGER NOT NULL,
///     time_updated INTEGER NOT NULL
/// );
/// CREATE TABLE IF NOT EXISTS session_message (
///     id TEXT PRIMARY KEY,
///     session_id TEXT NOT NULL,
///     seq INTEGER NOT NULL,
///     type TEXT NOT NULL,
///     time_created INTEGER NOT NULL,
///     data TEXT NOT NULL,
///     FOREIGN KEY (session_id) REFERENCES session(id)
/// );
/// CREATE TABLE IF NOT EXISTS session_context_epoch (
///     session_id TEXT PRIMARY KEY,
///     baseline_seq INTEGER NOT NULL
/// );
/// ```
pub struct SqliteSessionStore {
    conn: Arc<Mutex<rusqlite::Connection>>,
}

impl SqliteSessionStore {
    pub fn new(path: &str) -> Result<Self, rusqlite::Error> {
        let conn = rusqlite::Connection::open(path)?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS session (
                id TEXT PRIMARY KEY,
                data TEXT NOT NULL,
                time_created INTEGER NOT NULL,
                time_updated INTEGER NOT NULL
            );
            CREATE TABLE IF NOT EXISTS session_message (
                id TEXT PRIMARY KEY,
                session_id TEXT NOT NULL,
                seq INTEGER NOT NULL,
                type TEXT NOT NULL,
                time_created INTEGER NOT NULL,
                data TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_session_message_session ON session_message(session_id);
            CREATE INDEX IF NOT EXISTS idx_session_message_seq ON session_message(session_id, seq);
            CREATE TABLE IF NOT EXISTS session_context_epoch (
                session_id TEXT PRIMARY KEY,
                baseline_seq INTEGER NOT NULL
            );",
        )?;
        Ok(Self { conn: Arc::new(Mutex::new(conn)) })
    }

    pub fn in_memory() -> Result<Self, rusqlite::Error> {
        let conn = rusqlite::Connection::open_in_memory()?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS session (
                id TEXT PRIMARY KEY,
                data TEXT NOT NULL,
                time_created INTEGER NOT NULL,
                time_updated INTEGER NOT NULL
            );
            CREATE TABLE IF NOT EXISTS session_message (
                id TEXT PRIMARY KEY,
                session_id TEXT NOT NULL,
                seq INTEGER NOT NULL,
                type TEXT NOT NULL,
                time_created INTEGER NOT NULL,
                data TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_session_message_session ON session_message(session_id);
            CREATE INDEX IF NOT EXISTS idx_session_message_seq ON session_message(session_id, seq);
            CREATE TABLE IF NOT EXISTS session_context_epoch (
                session_id TEXT PRIMARY KEY,
                baseline_seq INTEGER NOT NULL
            );",
        )?;
        Ok(Self { conn: Arc::new(Mutex::new(conn)) })
    }
}

#[async_trait]
impl SessionStore for SqliteSessionStore {
    async fn get(&self, id: &SessionID) -> Option<SessionInfo> {
        let conn = self.conn.lock().ok()?;
        let result: rusqlite::Result<Option<SessionInfo>> = (|| {
            let mut stmt = conn.prepare("SELECT data FROM session WHERE id = ?1")?;
            let mut rows = stmt.query(rusqlite::params![id.as_str()])?;
            if let Some(row) = rows.next()? {
                let data: String = row.get(0)?;
                let info: SessionInfo = serde_json::from_str(&data).map_err(|e| rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e)))?;
                return Ok(Some(info));
            }
            Ok(None)
        })();
        result.ok().flatten()
    }

    async fn list(&self, limit: usize, offset: usize) -> Vec<SessionInfo> {
        let conn = match self.conn.lock() {
            Ok(conn) => conn,
            Err(_) => return Vec::new(),
        };
        let result: rusqlite::Result<Vec<SessionInfo>> = (|| {
            let mut stmt = conn.prepare(
                "SELECT data FROM session ORDER BY time_updated DESC LIMIT ?1 OFFSET ?2",
            )?;
            let rows = stmt.query_map(rusqlite::params![limit as i64, offset as i64], |row| {
                let data: String = row.get(0)?;
                Ok(data)
            })?;
            let mut infos = Vec::new();
            for row in rows {
                let data = row?;
                if let Ok(info) = serde_json::from_str::<SessionInfo>(&data) {
                    infos.push(info);
                }
            }
            Ok(infos)
        })();
        result.unwrap_or_default()
    }

    async fn create(&self, info: SessionInfo) -> SessionInfo {
        let conn = match self.conn.lock() {
            Ok(conn) => conn,
            Err(_) => return info,
        };
        let data = serde_json::to_string(&info).unwrap_or_default();
        let created = info.time.created.timestamp_millis();
        let updated = info.time.updated.timestamp_millis();
        let _ = conn.execute(
            "INSERT OR REPLACE INTO session (id, data, time_created, time_updated) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![info.id.as_str(), &data, created, updated],
        );
        info
    }

    async fn update(&self, info: SessionInfo) -> SessionInfo {
        let conn = match self.conn.lock() {
            Ok(conn) => conn,
            Err(_) => return info,
        };
        let data = serde_json::to_string(&info).unwrap_or_default();
        let updated = info.time.updated.timestamp_millis();
        let _ = conn.execute(
            "UPDATE session SET data = ?1, time_updated = ?2 WHERE id = ?3",
            rusqlite::params![&data, updated, info.id.as_str()],
        );
        info
    }

    async fn delete(&self, id: &SessionID) -> bool {
        let conn = match self.conn.lock() {
            Ok(conn) => conn,
            Err(_) => return false,
        };
        let _ = conn.execute(
            "DELETE FROM session_message WHERE session_id = ?1",
            rusqlite::params![id.as_str()],
        );
        conn.execute("DELETE FROM session WHERE id = ?1", rusqlite::params![id.as_str()])
            .map(|n| n > 0)
            .unwrap_or(false)
    }

    async fn context(&self, id: &SessionID) -> Option<Vec<SessionMessage>> {
        let conn = self.conn.lock().ok()?;
        let result: rusqlite::Result<Vec<SessionMessage>> = (|| {
            let mut stmt = conn.prepare(
                "SELECT data, type FROM session_message WHERE session_id = ?1 ORDER BY seq ASC",
            )?;
            let rows = stmt.query_map(rusqlite::params![id.as_str()], |row| {
                let data: String = row.get(0)?;
                let msg_type: String = row.get(1)?;
                Ok((data, msg_type))
            })?;
            let mut messages = Vec::new();
            for row in rows {
                let (data, msg_type) = row?;
                let mut wrapper = serde_json::from_str::<serde_json::Value>(&data).unwrap_or_default(); if let Some(obj) = wrapper.as_object_mut() { obj.insert("type".to_string(), serde_json::Value::String(msg_type)); } let wrapper = wrapper;
                if let Ok(msg) = serde_json::from_value::<SessionMessage>(wrapper) {
                    messages.push(msg);
                }
            }
            Ok(messages)
        })();
        Some(result.unwrap_or_default())
    }

    async fn append_message(&self, session_id: &SessionID, message: SessionMessage) -> bool {
        let conn = match self.conn.lock() {
            Ok(conn) => conn,
            Err(_) => return false,
        };
        let (msg_type, msg_id) = match &message {
            SessionMessage::User { id, .. } => ("user", id.0.as_str()),
            SessionMessage::Assistant { id, .. } => ("assistant", id.0.as_str()),
            SessionMessage::System { id, .. } => ("system", id.0.as_str()),
            SessionMessage::Shell { id, .. } => ("shell", id.0.as_str()),
            SessionMessage::Synthetic { id, .. } => ("synthetic", id.0.as_str()),
            SessionMessage::AgentSwitched { id, .. } => ("agent-switched", id.0.as_str()),
            SessionMessage::ModelSwitched { id, .. } => ("model-switched", id.0.as_str()),
            SessionMessage::Compaction { id, .. } => ("compaction", id.0.as_str()),
        };
        let data = serde_json::to_string(&message).unwrap_or_default();
        let now = chrono::Utc::now().timestamp_millis();
        let seq: i64 = conn
            .query_row(
                "SELECT COALESCE(MAX(seq), 0) + 1 FROM session_message WHERE session_id = ?1",
                rusqlite::params![session_id.as_str()],
                |row| row.get(0),
            )
            .unwrap_or(1);
        conn.execute(
            "INSERT OR REPLACE INTO session_message (id, session_id, seq, type, time_created, data) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![msg_id, session_id.as_str(), seq, msg_type, now, &data],
        )
        .map(|n| n > 0)
        .unwrap_or(false)
    }

    async fn get_message(&self, session_id: &SessionID, message_id: &str) -> Option<SessionMessage> {
        let conn = self.conn.lock().ok()?;
        let result: rusqlite::Result<Option<SessionMessage>> = (|| {
            let mut stmt = conn.prepare(
                "SELECT data, type FROM session_message WHERE id = ?1 AND session_id = ?2",
            )?;
            let mut rows = stmt.query(rusqlite::params![message_id, session_id.as_str()])?;
            if let Some(row) = rows.next()? {
                let data: String = row.get(0)?;
                let msg_type: String = row.get(1)?;
                let mut wrapper = serde_json::from_str::<serde_json::Value>(&data).unwrap_or_default(); if let Some(obj) = wrapper.as_object_mut() { obj.insert("type".to_string(), serde_json::Value::String(msg_type)); } let wrapper = wrapper;
                if let Ok(msg) = serde_json::from_value::<SessionMessage>(wrapper) {
                    return Ok(Some(msg));
                }
            }
            Ok(None)
        })();
        result.ok().flatten()
    }
}
