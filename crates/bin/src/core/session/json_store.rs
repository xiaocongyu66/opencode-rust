//! File-based session store using one JSON file per session.
//!
//! Sessions are stored at `~/.rsopencode/sessions/<session_id>.json` in a
//! human-readable, easy-to-edit format:
//!
//! ```json
//! {
//!   "info": { "id": "ses_...", "title": "New session", ... },
//!   "messages": [
//!     { "id": "msg_1", "type": "user", "text": "hello", ... },
//!     { "id": "msg_2", "type": "assistant", "content": [...], ... }
//!   ]
//! }
//! ```

use std::path::PathBuf;
use std::sync::Mutex;

use async_trait::async_trait;

use crate::schema::session::{SessionInfo, SessionMessage};
use crate::schema::ids::SessionID;

use super::SessionStore;

/// File-backed session store. Each session lives in its own JSON file.
pub struct JsonSessionStore {
    dir: PathBuf,
    /// Serializes writes per-session to avoid concurrent-append corruption.
    _write_lock: Mutex<()>,
}

impl JsonSessionStore {
    pub fn new(dir: impl Into<PathBuf>) -> std::io::Result<Self> {
        let dir = dir.into();
        std::fs::create_dir_all(&dir)?;
        Ok(Self {
            dir,
            _write_lock: Mutex::new(()),
        })
    }

    /// Create the default store at `~/.rsopencode/sessions/`.
    pub fn default_dir() -> Option<PathBuf> {
        dirs::home_dir().map(|h| h.join(".rsopencode").join("sessions"))
    }

    fn session_path(&self, id: &SessionID) -> PathBuf {
        // Sanitize the id to a safe filename (strip any path separators).
        let safe = id.0.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_");
        self.dir.join(format!("{safe}.json"))
    }

    fn read_session(&self, id: &SessionID) -> Option<StoredSession> {
        let path = self.session_path(id);
        let content = std::fs::read_to_string(&path).ok()?;
        serde_json::from_str(&content).ok()
    }

    fn write_session(&self, stored: &StoredSession) -> std::io::Result<()> {
        let _lock = self._write_lock.lock().unwrap();
        let path = self.session_path(&stored.info.id);
        let json = serde_json::to_string_pretty(stored)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        std::fs::write(&path, json)
    }

    fn list_session_ids(&self) -> Vec<String> {
        let mut ids = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&self.dir) {
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    if let Some(stem) = name.strip_suffix(".json") {
                        ids.push(stem.to_string());
                    }
                }
            }
        }
        ids
    }
}

/// On-disk representation: session info + ordered messages.
#[derive(serde::Serialize, serde::Deserialize)]
struct StoredSession {
    info: SessionInfo,
    messages: Vec<SessionMessage>,
}

#[async_trait]
impl SessionStore for JsonSessionStore {
    async fn get(&self, id: &SessionID) -> Option<SessionInfo> {
        self.read_session(id).map(|s| s.info)
    }

    async fn list(&self, limit: usize, offset: usize) -> Vec<SessionInfo> {
        let mut infos: Vec<SessionInfo> = Vec::new();
        for id_str in self.list_session_ids() {
            let sid = SessionID(id_str);
            if let Some(stored) = self.read_session(&sid) {
                infos.push(stored.info);
            }
        }
        // Sort by updated time, newest first.
        infos.sort_by(|a, b| b.time.updated.cmp(&a.time.updated));
        infos.into_iter().skip(offset).take(limit).collect()
    }

    async fn create(&self, info: SessionInfo) -> SessionInfo {
        let stored = StoredSession {
            info: info.clone(),
            messages: Vec::new(),
        };
        let _ = self.write_session(&stored);
        info
    }

    async fn update(&self, info: SessionInfo) -> SessionInfo {
        // Load existing messages, keep them, replace the info.
        let mut stored = self.read_session(&info.id).unwrap_or(StoredSession {
            info: info.clone(),
            messages: Vec::new(),
        });
        stored.info = info.clone();
        let _ = self.write_session(&stored);
        info
    }

    async fn delete(&self, id: &SessionID) -> bool {
        let path = self.session_path(id);
        std::fs::remove_file(&path).is_ok()
    }

    async fn context(&self, id: &SessionID) -> Option<Vec<SessionMessage>> {
        self.read_session(id).map(|s| s.messages)
    }

    async fn append_message(&self, session_id: &SessionID, message: SessionMessage) -> bool {
        let mut stored = match self.read_session(session_id) {
            Some(s) => s,
            None => return false,
        };
        stored.messages.push(message);
        stored.info.time.updated = chrono::Utc::now();
        self.write_session(&stored).is_ok()
    }

    async fn get_message(&self, session_id: &SessionID, message_id: &str) -> Option<SessionMessage> {
        let stored = self.read_session(session_id)?;
        stored
            .messages
            .into_iter()
            .find(|m| match m {
                SessionMessage::User { id, .. } => id.0 == message_id,
                SessionMessage::Assistant { id, .. } => id.0 == message_id,
                SessionMessage::System { id, .. } => id.0 == message_id,
                SessionMessage::Shell { id, .. } => id.0 == message_id,
                SessionMessage::Synthetic { id, .. } => id.0 == message_id,
                SessionMessage::Compaction { id, .. } => id.0 == message_id,
                SessionMessage::AgentSwitched { id, .. } => id.0 == message_id,
                SessionMessage::ModelSwitched { id, .. } => id.0 == message_id,
            })
            .map(|m| m)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::session::{SessionTokens, SessionTime};
    use crate::schema::ids::{ProjectID, AgentID};
    use crate::schema::location::LocationRef;
    use crate::schema::common::AbsolutePath;

    fn temp_store() -> JsonSessionStore {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let n = COUNTER.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!("rsopencode-test-{}-{}", std::process::id(), n));
        let _ = std::fs::remove_dir_all(&dir);
        JsonSessionStore::new(&dir).expect("create store")
    }

    fn make_info(id: &str) -> SessionInfo {
        SessionInfo {
            id: SessionID(id.to_string()),
            parent_id: None,
            project_id: ProjectID::from_str("default"),
            agent: Some(AgentID("build".to_string())),
            model: None,
            cost: 0.0,
            tokens: SessionTokens::default(),
            time: SessionTime {
                created: chrono::Utc::now(),
                updated: chrono::Utc::now(),
                archived: None,
            },
            title: "Test session".to_string(),
            location: LocationRef {
                directory: AbsolutePath("/tmp".to_string()),
                workspace_id: None,
            },
            subpath: None,
            revert: None,
        }
    }

    #[tokio::test]
    async fn create_and_get_session() {
        let store = temp_store();
        let info = make_info("ses_test1");
        store.create(info.clone()).await;
        let got = store.get(&SessionID("ses_test1".to_string())).await;
        assert!(got.is_some());
        assert_eq!(got.unwrap().id.0, "ses_test1");
    }

    #[tokio::test]
    async fn append_message_and_read_context() {
        let store = temp_store();
        let sid = SessionID("ses_test2".to_string());
        store.create(make_info("ses_test2")).await;

        let msg = SessionMessage::User {
            id: crate::schema::ids::SessionMessageID::new(),
            metadata: None,
            time: crate::schema::session::MessageTime {
                created: chrono::Utc::now(),
            },
            text: "hello world".to_string(),
            files: None,
            agents: None,
        };
        let ok = store.append_message(&sid, msg).await;
        assert!(ok);

        let ctx = store.context(&sid).await.expect("context");
        assert_eq!(ctx.len(), 1);
    }

    #[tokio::test]
    async fn list_returns_all_sessions() {
        let store = temp_store();
        store.create(make_info("ses_a")).await;
        store.create(make_info("ses_b")).await;
        let all = store.list(100, 0).await;
        assert_eq!(all.len(), 2);
    }

    #[tokio::test]
    async fn delete_removes_session() {
        let store = temp_store();
        store.create(make_info("ses_del")).await;
        assert!(store.delete(&SessionID("ses_del".to_string())).await);
        assert!(store.get(&SessionID("ses_del".to_string())).await.is_none());
    }

    #[tokio::test]
    async fn missing_session_returns_none() {
        let store = temp_store();
        assert!(store.get(&SessionID("nonexistent".to_string())).await.is_none());
    }
}
