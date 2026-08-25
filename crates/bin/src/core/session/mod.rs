//! Session management — the core orchestration module.
//!
//! Ported from `session/` directory in the TS original.

pub mod json_store;
pub mod store;
pub mod history;
pub mod prompt;
pub mod compaction;
pub mod execution;
pub mod projector;
pub mod runner;
pub mod schema;
pub mod status;
pub mod retry;
pub mod overflow;
pub mod run_state;
pub mod revert;
pub mod tools;
pub mod todo;
pub mod instruction;
pub mod system;
pub mod summary;
pub mod message_v2;
pub mod processor;
pub mod llm;
pub mod reminders;

use async_trait::async_trait;
use crate::schema::ids::SessionID;
use crate::schema::session::{SessionInfo, SessionMessage};

#[async_trait]
pub trait SessionStore: Send + Sync {
    async fn get(&self, id: &SessionID) -> Option<SessionInfo>;
    async fn list(&self, limit: usize, offset: usize) -> Vec<SessionInfo>;
    async fn create(&self, info: SessionInfo) -> SessionInfo;
    async fn update(&self, info: SessionInfo) -> SessionInfo;
    async fn delete(&self, id: &SessionID) -> bool;
    async fn context(&self, id: &SessionID) -> Option<Vec<SessionMessage>>;
    async fn append_message(&self, session_id: &SessionID, message: SessionMessage) -> bool;
    async fn get_message(&self, session_id: &SessionID, message_id: &str) -> Option<SessionMessage>;
}

/// Default title prefixes.
pub const PARENT_TITLE_PREFIX: &str = "New session - ";
pub const CHILD_TITLE_PREFIX: &str = "Child session - ";

/// Generate a default parent-session title with the current ISO timestamp.
/// Mirrors the TS `New session - ${new Date(now).toISOString()}`.
pub fn default_parent_title() -> String {
    format!("{}{}", PARENT_TITLE_PREFIX, chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true))
}

/// Generate a default child-session title with the current ISO timestamp.
pub fn default_child_title() -> String {
    format!("{}{}", CHILD_TITLE_PREFIX, chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true))
}

/// Check if a title is a default (auto-generated) title.
pub fn is_default_title(title: &str) -> bool {
    let pattern = format!(
        r"^({}|{})\d{{4}}-\d{{2}}-\d{{2}}T\d{{2}}:\d{{2}}:\d{{2}}\.\d{{3}}Z$",
        regex::escape(PARENT_TITLE_PREFIX),
        regex::escape(CHILD_TITLE_PREFIX)
    );
    regex::Regex::new(&pattern)
        .map(|re| re.is_match(title))
        .unwrap_or(false)
}

/// Generate a forked title.
pub fn forked_title(title: &str) -> String {
    let re = regex::Regex::new(r"^(.+) \(fork #(\d+)\)$").unwrap();
    if let Some(caps) = re.captures(title) {
        let base = caps.get(1).unwrap().as_str();
        let num: u32 = caps.get(2).unwrap().as_str().parse().unwrap_or(0);
        return format!("{} (fork #{})", base, num + 1);
    }
    format!("{} (fork #1)", title)
}

/// Calculate the relative path from worktree to cwd.
pub fn session_path(worktree: &str, cwd: &str) -> String {
    let worktree_abs = std::path::Path::new(worktree)
        .canonicalize()
        .unwrap_or_else(|_| std::path::PathBuf::from(worktree));
    let cwd_abs = std::path::Path::new(cwd)
        .canonicalize()
        .unwrap_or_else(|_| std::path::PathBuf::from(cwd));
    cwd_abs
        .strip_prefix(&worktree_abs)
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .unwrap_or_default()
}
