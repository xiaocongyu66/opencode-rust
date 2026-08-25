//! Hook protocol — JSON input/output for hook commands.
//!
//! Follows claude-code-book Ch08: hooks receive a JSON payload on stdin
//! and respond with a JSON decision on stdout. Five hook types share the
//! same protocol: Command (shell), Prompt (LLM), Agent (multi-step LLM),
//! HTTP (POST), Function (in-process).

use serde::{Deserialize, Serialize};

/// 26 lifecycle event names where hooks can attach (claude-code-book Ch08).
/// Grouped: session lifecycle, tool lifecycle, prompt lifecycle, compact.
pub const EVENT_SESSION_START: &str = "SessionStart";
pub const EVENT_SESSION_END: &str = "SessionEnd";
pub const EVENT_USER_PROMPT_SUBMIT: &str = "UserPromptSubmit";
pub const EVENT_PRE_TOOL_USE: &str = "PreToolUse";
pub const EVENT_POST_TOOL_USE: &str = "PostToolUse";
pub const EVENT_PRE_COMPACT: &str = "PreCompact";
pub const EVENT_POST_COMPACT: &str = "PostCompact";
pub const EVENT_NOTIFICATION: &str = "Notification";
pub const EVENT_STOP: &str = "Stop";
pub const EVENT_SUBAGENT_STOP: &str = "SubagentStop";
pub const EVENT_PRE_FILE_EDIT: &str = "PreFileEdit";

/// All known event names for validation.
pub const ALL_EVENTS: &[&str] = &[
    EVENT_SESSION_START,
    EVENT_SESSION_END,
    EVENT_USER_PROMPT_SUBMIT,
    EVENT_PRE_TOOL_USE,
    EVENT_POST_TOOL_USE,
    EVENT_PRE_COMPACT,
    EVENT_POST_COMPACT,
    EVENT_NOTIFICATION,
    EVENT_STOP,
    EVENT_SUBAGENT_STOP,
    EVENT_PRE_FILE_EDIT,
];

/// The JSON payload passed to a hook on stdin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookInput {
    /// Event name (e.g. "PreToolUse").
    pub event: String,
    /// Tool name for tool-related events (None for session/prompt events).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool: Option<String>,
    /// The raw tool input or user prompt, as JSON or text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input: Option<serde_json::Value>,
    /// Session ID for correlation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    /// Working directory at the time of the event.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
}

/// The decision a hook returns on stdout.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HookOutput {
    /// "allow" | "deny" | "ask". Empty means no decision (passthrough).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decision: Option<String>,
    /// Human-readable reason shown to the user when denying.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    /// Replace the tool input with this value (PreToolUse only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_input: Option<serde_json::Value>,
    /// Inject extra context into the conversation (UserPromptSubmit only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_context: Option<String>,
}

/// Simplified decision for callers that don't need the full output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HookDecision {
    /// Hook explicitly allowed the action.
    Allow,
    /// Hook denied the action; `reason` should be shown to the user.
    Deny,
    /// Hook asks for user confirmation.
    Ask,
    /// Hook returned no decision; proceed with default behavior.
    Passthrough,
}

impl HookOutput {
    pub fn decision(&self) -> HookDecision {
        match self.decision.as_deref() {
            Some("allow") => HookDecision::Allow,
            Some("deny") => HookDecision::Deny,
            Some("ask") => HookDecision::Ask,
            _ => HookDecision::Passthrough,
        }
    }
}
