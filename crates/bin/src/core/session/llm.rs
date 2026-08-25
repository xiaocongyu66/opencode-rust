//! LLM streaming interface.
//!
//! Ported from `session/llm.ts`.
//! Provides the streaming interface for LLM calls.

use std::collections::HashMap;

use crate::schema::ids::{ProviderID, SessionID};

/// Stream input for LLM calls.
#[derive(Debug, Clone)]
pub struct StreamInput {
    pub user: crate::schema::session::SessionMessage,
    pub session_id: SessionID,
    pub parent_session_id: Option<SessionID>,
    pub model: ModelRef,
    pub agent_name: String,
    pub system: Vec<String>,
    pub messages: Vec<ModelMessage>,
    pub small: bool,
    pub tools: HashMap<String, serde_json::Value>,
    pub retries: u32,
    pub tool_choice: Option<ToolChoice>,
}

/// Model reference.
#[derive(Debug, Clone)]
pub struct ModelRef {
    pub provider_id: ProviderID,
    pub model_id: String,
    pub variant: Option<String>,
}

/// Model message types.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "role")]
pub enum ModelMessage {
    #[serde(rename = "user")]
    User { content: String },
    #[serde(rename = "assistant")]
    Assistant { content: String },
    #[serde(rename = "system")]
    System { content: String },
}

/// Tool choice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolChoice {
    Auto,
    Required,
    None,
}

/// Default max output tokens.
pub const OUTPUT_TOKEN_MAX: u64 = 32_000;

/// Check if a message has tool calls.
pub fn has_tool_calls(message: &crate::schema::session::SessionMessage) -> bool {
    if let crate::schema::session::SessionMessage::Assistant { content, .. } = message {
        content.iter().any(|item| {
            matches!(item, crate::schema::session::AssistantContent::Tool { .. })
        })
    } else {
        false
    }
}
