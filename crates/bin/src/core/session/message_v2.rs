//! Session message handling (v2).
//!
//! Ported from `session/message-v2.ts`.
//! Handles message pagination, model message conversion, and error parsing.


use crate::schema::session::{SessionMessage, SessionMessageUnknownError};

/// Synthetic attachment prompt for media from tool results.
pub const SYNTHETIC_ATTACHMENT_PROMPT: &str = "Attached media from tool result:";

/// Cursor for pagination.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Cursor {
    pub id: String,
    pub time: f64,
}

impl Cursor {
    pub fn encode(&self) -> String {
        let json = serde_json::to_string(self).unwrap_or_default();
        use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
        URL_SAFE_NO_PAD.encode(json)
    }

    pub fn decode(input: &str) -> Option<Self> {
        use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
        let json = URL_SAFE_NO_PAD.decode(input).ok()?;
        serde_json::from_slice(&json).ok()
    }
}

/// Paginated message result.
#[derive(Debug, Clone)]
pub struct PageResult {
    pub items: Vec<SessionMessage>,
    pub more: bool,
    pub cursor: Option<String>,
}

/// Check if a MIME type is media.
pub fn is_media(mime: &str) -> bool {
    mime.starts_with("image/")
        || mime.starts_with("audio/")
        || mime.starts_with("video/")
        || mime == "application/pdf"
}

/// Truncate tool output for compaction.
pub fn truncate_tool_output(text: &str, max_chars: Option<usize>) -> String {
    let max = max_chars.unwrap_or(2000);
    if text.len() <= max {
        return text.to_string();
    }
    let omitted = text.len() - max;
    let truncated = if max <= text.len() { &text[..max] } else { text };
    format!(
        "{}\n[Tool output truncated for compaction: omitted {} chars]",
        truncated,
        omitted
    )
}

/// Parse an error into a structured error object.
pub fn from_error(
    e: &dyn std::error::Error,
    provider_id: &str,
    _aborted: bool,
) -> SessionMessageUnknownError {
    let message = e.to_string();
    if message.contains("Aborted") || message.contains("AbortError") {
        return SessionMessageUnknownError {
            error_type: "aborted".to_string(),
            message,
        };
    }

    if message.contains("context")
        && (message.contains("overflow") || message.contains("exceeds"))
    {
        return SessionMessageUnknownError {
            error_type: "context_overflow".to_string(),
            message,
        };
    }

    if message.contains("auth")
        || message.contains("unauthorized")
        || message.contains("api key")
    {
        return SessionMessageUnknownError {
            error_type: "auth".to_string(),
            message: format!("Provider {}: {}", provider_id, message),
        };
    }

    if message.contains("connection reset")
        || message.contains("ECONNRESET")
        || message.contains("timeout")
    {
        return SessionMessageUnknownError {
            error_type: "api".to_string(),
            message,
        };
    }

    SessionMessageUnknownError {
        error_type: "unknown".to_string(),
        message,
    }
}

/// Find the latest user and assistant messages.
pub fn latest(messages: &[SessionMessage]) -> (Option<SessionMessage>, Option<SessionMessage>, Option<SessionMessage>) {
    let mut user: Option<SessionMessage> = None;
    let mut assistant: Option<SessionMessage> = None;
    let mut finished: Option<SessionMessage> = None;

    for msg in messages {
        match msg {
            SessionMessage::User { .. } => {
                user = Some(msg.clone());
            }
            SessionMessage::Assistant { finish, .. } => {
                assistant = Some(msg.clone());
                if finish.is_some() {
                    finished = Some(msg.clone());
                }
            }
            _ => {}
        }
    }

    (user, assistant, finished)
}

/// Filter compacted messages for model consumption.
pub fn filter_compacted(messages: Vec<SessionMessage>) -> Vec<SessionMessage> {
    // In the full implementation this reorders messages around compaction boundaries
    messages
}
