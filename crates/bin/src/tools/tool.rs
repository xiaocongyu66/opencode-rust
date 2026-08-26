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

/// Permission decision returned by `check_permissions` (claude-code-book Ch04).
/// Three layers: input validation, rule matching, context evaluation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PermissionDecision {
    /// Allow the tool call to proceed.
    Allow,
    /// Deny with a human-readable reason; shown to the user.
    Deny(String),
    /// Ask the user for confirmation (interactive prompt).
    Ask,
}

/// Context modifier — lets a tool tweak the execution context for downstream
/// tool calls within the same turn (claude-code-book Ch03 "contextModifier").
#[derive(Debug, Clone, Default)]
pub struct ContextModifier {
    /// Override the working directory for subsequent tools.
    pub cwd: Option<std::path::PathBuf>,
    /// Extra env vars to inject.
    pub env: Vec<(String, String)>,
}

/// The core tool trait. Every built-in tool implements this.
///
/// Five elements per claude-code-book Ch03:
/// 1. name + aliases — identity (rename is "add-only" via aliases)
/// 2. parameters_schema — serde/JSON Schema for validation + API docs
/// 3. permission model — validate_input → check_permissions, plus
///    is_concurrency_safe for scheduler partitioning
/// 4. execute + context_modifier — core logic + context side-effect
/// 5. UI render methods — six lifecycle hooks for the TUI
///
/// All non-execute methods have default impls so existing 40+ tools keep
/// working. Override the ones that matter per tool.
#[async_trait]
pub trait Tool: Send + Sync {
    // --- Element 1: name + aliases ---

    fn name(&self) -> &str;

    /// Alternate names for backward compatibility. When a tool is renamed,
    /// the old name stays here so existing configs/scripts still work.
    /// Default: no aliases.
    fn aliases(&self) -> &[&str] {
        &[]
    }

    fn description(&self) -> &str;

    // --- Element 2: schema ---

    fn parameters_schema(&self) -> serde_json::Value;

    // --- Element 3: permission model (three layers, Ch04) ---

    /// Layer 1: validate the parsed input before permission checks.
    /// Return Err(message) to reject malformed input early (fail fast).
    /// Default: accept everything.
    fn validate_input(&self, _params: &serde_json::Value) -> Result<(), String> {
        Ok(())
    }

    /// Layer 2-3: tool-specific permission logic. The dispatcher runs
    /// rule-matching first (deny > ask > allow), then calls this for
    /// context-aware evaluation. Default: passthrough (ask the user).
    fn check_permissions(&self, _params: &serde_json::Value) -> PermissionDecision {
        PermissionDecision::Ask
    }

    /// Whether this tool can run in parallel with other concurrency-safe
    /// tools (Ch03 "concurrency partition"). Default: false (serial).
    fn is_concurrency_safe(&self) -> bool {
        false
    }

    // --- Element 4: execute + context_modifier ---

    async fn execute(
        &self,
        params: serde_json::Value,
        ctx: &ToolContext,
    ) -> Result<ToolResult, ToolFailure>;

    /// Optional context modifier applied after this tool runs, affecting
    /// subsequent tools in the same turn. Default: no modification.
    fn context_modifier(&self) -> Option<ContextModifier> {
        None
    }

    // --- Element 5: UI render methods (six lifecycle hooks, Ch03) ---

    /// Render the tool call before it starts (pending state).
    fn render_pending(&self, _input: &serde_json::Value) -> Option<String> {
        None
    }

    /// Render while the tool is running (spinner/progress).
    fn render_running(&self, _input: &serde_json::Value) -> Option<String> {
        None
    }

    /// Render a successful result summary.
    fn render_completed(&self, _result: &ToolResult) -> Option<String> {
        None
    }

    /// Render a failure summary.
    fn render_failed(&self, _error: &ToolFailure) -> Option<String> {
        None
    }

    /// Render the tool-call bubble header (e.g. "⚙ Bash").
    fn render_tool_call(&self) -> Option<String> {
        None
    }

    /// Render the tool-result bubble footer (e.g. "[exit: 0]").
    fn render_tool_result(&self, _result: &ToolResult) -> Option<String> {
        None
    }
}
