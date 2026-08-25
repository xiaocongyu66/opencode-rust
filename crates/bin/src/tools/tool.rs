//! Tool trait and context — the core abstraction for LLM-callable tools.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Execution context passed to a tool handler.
#[derive(Debug, Clone)]
pub struct ToolContext {
    pub session_id: String,
    pub agent_id: String,
    pub assistant_message_id: String,
    pub tool_call_id: String,
}

/// Tool output content — text or file.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ToolContent {
    Text { text: String },
    File { data: String, mime: String, #[serde(skip_serializing_if = "Option::is_none")] name: Option<String> },
}

/// Structured tool result.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ToolResult {
    pub output: String,
    pub structured: serde_json::Value,
    pub content: Vec<ToolContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_paths: Option<Vec<String>>,
}

impl ToolResult {
    pub fn text(text: impl Into<String>) -> Self {
        let text = text.into();
        Self {
            output: text.clone(),
            structured: serde_json::Value::Null,
            content: vec![ToolContent::Text { text }],
            output_paths: None,
        }
    }
}

/// Tool execution error.
#[derive(Debug, thiserror::Error)]
pub enum ToolFailure {
    #[error("{0}")]
    Message(String),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error("Task failed: {0}")]
    TaskJoin(String),
}

/// The core tool trait. Every built-in tool implements this.
#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn parameters_schema(&self) -> serde_json::Value;

    async fn execute(
        &self,
        params: serde_json::Value,
        ctx: &ToolContext,
    ) -> Result<ToolResult, ToolFailure>;
}
