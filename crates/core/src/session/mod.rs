//! Session management — the core orchestration module.

pub mod store;
pub mod history;
pub mod prompt;
pub mod compaction;
pub mod execution;
pub mod projector;
pub mod runner;
pub mod message_converter;

use async_trait::async_trait;
use opencode_schema::ids::SessionID;
use opencode_schema::session::{SessionInfo, SessionMessage};

/// Session store trait — persists sessions and their data.
#[async_trait]
pub trait SessionStore: Send + Sync {
    async fn get(&self, id: &SessionID) -> Option<SessionInfo>;
    async fn list(&self, limit: usize, offset: usize) -> Vec<SessionInfo>;
    async fn create(&self, info: SessionInfo) -> SessionInfo;
    async fn update(&self, info: SessionInfo) -> SessionInfo;
    async fn delete(&self, id: &SessionID) -> bool;

    /// Get the active context messages for a session (all messages after last compaction).
    async fn context(&self, id: &SessionID) -> Option<Vec<SessionMessage>>;

    /// Append a message to the session.
    async fn append_message(&self, session_id: &SessionID, message: SessionMessage) -> bool;

    /// Get a specific message by ID.
    async fn get_message(&self, session_id: &SessionID, message_id: &str) -> Option<SessionMessage>;
}
