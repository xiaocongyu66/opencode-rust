//! Session store — in-memory implementation.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use async_trait::async_trait;
use opencode_schema::ids::SessionID;
use opencode_schema::session::{SessionInfo, SessionMessage};
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
