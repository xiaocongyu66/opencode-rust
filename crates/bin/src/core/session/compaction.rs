//! Session compaction logic.
//!
//! Ported from `session/compaction.ts`.
//! Manages context window compaction by summarizing old messages
//! and pruning tool outputs.

use std::sync::Arc;

use tokio::sync::RwLock;

use crate::schema::ids::SessionID;
use crate::schema::session::SessionMessage;

/// Minimum tokens to prune.
pub const PRUNE_MINIMUM: u64 = 20_000;
/// Token threshold before pruning begins.
pub const PRUNE_PROTECT: u64 = 40_000;
/// Max characters for tool output during compaction.
pub const TOOL_OUTPUT_MAX_CHARS: usize = 2_000;
/// Tools whose outputs are never pruned.
pub const PRUNE_PROTECTED_TOOLS: &[&str] = &["skill"];
/// Min tokens to preserve in recent context.
pub const MIN_PRESERVE_RECENT_TOKENS: u64 = 2_000;
/// Max tokens to preserve in recent context.
pub const MAX_PRESERVE_RECENT_TOKENS: u64 = 15_000;

/// Compaction event types.
pub const EVENT_COMPACTION_STARTED: &str = "session.compaction.started";
pub const EVENT_COMPACTION_ENDED: &str = "session.compaction.ended";
pub const EVENT_COMPACTED: &str = "session.compacted";

/// Result of a compaction process.
#[derive(Debug, Clone, PartialEq)]
pub enum CompactionResult {
    Continue,
    Stop,
    Compact,
}

/// A turn boundary in the conversation.
#[derive(Debug, Clone)]
pub struct Turn {
    pub start: usize,
    pub end: usize,
    pub id: String,
}

/// A tail reference for compaction.
#[derive(Debug, Clone)]
pub struct Tail {
    pub start: usize,
    pub id: String,
}

/// State for compaction selection.
#[derive(Debug, Clone, Default)]
pub struct SelectionResult {
    pub head: Vec<usize>,
    pub tail_start_id: Option<String>,
}

/// Serialize a message for compaction prompt.
pub fn serialize_message(msg: &SessionMessage) -> String {
    match msg {
        SessionMessage::User { text, .. } => {
            if text.is_empty() {
                String::new()
            } else {
                format!("[User]: {}", text)
            }
        }
        SessionMessage::Assistant { content, .. } => {
            let mut parts = Vec::new();
            for item in content {
                match item {
                    crate::schema::session::AssistantContent::Text { text, .. } => {
                        if !text.is_empty() {
                            parts.push(format!("[Assistant]: {}", text));
                        }
                    }
                    crate::schema::session::AssistantContent::Reasoning { text, .. } => {
                        if !text.is_empty() {
                            parts.push(format!("[Assistant reasoning]: {}", text));
                        }
                    }
                    crate::schema::session::AssistantContent::Tool { name, state, .. } => {
                        let call = format!("[Assistant tool call]: {}", name);
                        parts.push(call);
                        match state {
                            crate::schema::session::ToolState::Completed { .. } => {
                                parts.push("[Tool result]: [completed]".to_string());
                            }
                            crate::schema::session::ToolState::Error { error, .. } => {
                                parts.push(format!("[Tool error]: {}", error.message));
                            }
                            _ => {}
                        }
                    }
                }
            }
            parts.join("\n")
        }
        SessionMessage::Compaction { summary, .. } => {
            format!("[Compaction summary]: {}", summary)
        }
        SessionMessage::Synthetic { text, .. } => {
            format!("[System]: {}", text)
        }
        SessionMessage::System { text, .. } => {
            format!("[System]: {}", text)
        }
        SessionMessage::Shell { command, output, .. } => {
            format!("[Shell]: {}\n{}", command, output)
        }
        _ => String::new(),
    }
}

/// Truncate tool output to max chars.
pub fn truncate_tool_output(value: &str) -> String {
    if value.len() <= TOOL_OUTPUT_MAX_CHARS {
        return value.to_string();
    }
    format!(
        "{}\n[truncated]",
        &value[..TOOL_OUTPUT_MAX_CHARS.min(value.len())]
    )
}

/// Identify turns in a message list.
pub fn turns(messages: &[SessionMessage]) -> Vec<Turn> {
    let mut result = Vec::new();
    for (i, msg) in messages.iter().enumerate() {
        if !matches!(msg, SessionMessage::User { .. }) {
            continue;
        }
        if matches!(msg, SessionMessage::Compaction { .. }) {
            continue;
        }
        result.push(Turn {
            start: i,
            end: messages.len(),
            id: match msg {
                SessionMessage::User { id, .. } => id.to_string(),
                _ => String::new(),
            },
        });
    }
    for i in 0..result.len().saturating_sub(1) {
        result[i].end = result[i + 1].start;
    }
    result
}

/// Session compaction manager.
pub struct SessionCompactionManager {
    state: Arc<RwLock<CompactionState>>,
}

#[derive(Debug, Default)]
pub struct CompactionState {
    pub active: bool,
    pub session_id: Option<SessionID>,
}

impl SessionCompactionManager {
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(CompactionState::default())),
        }
    }

    pub async fn is_active(&self) -> bool {
        self.state.read().await.active
    }

    pub async fn set_active(&self, session_id: Option<SessionID>) {
        let mut state = self.state.write().await;
        state.active = session_id.is_some();
        state.session_id = session_id;
    }
}

impl Default for SessionCompactionManager {
    fn default() -> Self {
        Self::new()
    }
}
