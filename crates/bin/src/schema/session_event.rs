//! Session event types — the largest schema module.
//!
//! This module defines all durable and live events for sessions.
//! In the TS original, events are defined via a factory pattern; here
//! they are represented as Rust enums with serde tag = "type".

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::schema::common::RelativePath;
use crate::schema::ids::{SessionID, SessionMessageID};
use crate::schema::llm::{ProviderMetadata, ToolContent};
use crate::schema::location::LocationRef;
use crate::schema::model::ModelRef;
use crate::schema::prompt::Prompt;
use crate::schema::revert::RevertState;
use crate::schema::session::{CompactionReason, SessionDelivery, SessionMessageUnknownError, SessionStatus};

/// Session event — tagged union of all session event types.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum SessionEvent {
    #[serde(rename = "session.next.agent.switched")]
    AgentSwitched(SessionAgentSwitched),

    #[serde(rename = "session.next.model.switched")]
    ModelSwitched(SessionModelSwitched),

    #[serde(rename = "session.next.moved")]
    Moved(SessionMoved),

    #[serde(rename = "session.next.prompted")]
    Prompted(SessionPrompted),

    #[serde(rename = "session.next.prompt.admitted")]
    PromptAdmitted(SessionPromptAdmitted),

    #[serde(rename = "session.next.context.updated")]
    ContextUpdated(SessionContextUpdated),

    #[serde(rename = "session.next.synthetic")]
    Synthetic(SessionSynthetic),

    #[serde(rename = "session.next.shell.started")]
    ShellStarted(SessionShellStarted),

    #[serde(rename = "session.next.shell.ended")]
    ShellEnded(SessionShellEnded),

    #[serde(rename = "session.next.step.started")]
    StepStarted(SessionStepStarted),

    #[serde(rename = "session.next.step.ended")]
    StepEnded(SessionStepEnded),

    #[serde(rename = "session.next.step.failed")]
    StepFailed(SessionStepFailed),

    #[serde(rename = "session.next.text.started")]
    TextStarted(SessionTextStarted),

    #[serde(rename = "session.next.text.delta")]
    TextDelta(SessionTextDelta),

    #[serde(rename = "session.next.text.ended")]
    TextEnded(SessionTextEnded),

    #[serde(rename = "session.next.reasoning.started")]
    ReasoningStarted(SessionReasoningStarted),

    #[serde(rename = "session.next.reasoning.delta")]
    ReasoningDelta(SessionReasoningDelta),

    #[serde(rename = "session.next.reasoning.ended")]
    ReasoningEnded(SessionReasoningEnded),

    #[serde(rename = "session.next.tool.input.started")]
    ToolInputStarted(SessionToolInputStarted),

    #[serde(rename = "session.next.tool.input.delta")]
    ToolInputDelta(SessionToolInputDelta),

    #[serde(rename = "session.next.tool.input.ended")]
    ToolInputEnded(SessionToolInputEnded),

    #[serde(rename = "session.next.tool.called")]
    ToolCalled(SessionToolCalled),

    #[serde(rename = "session.next.tool.progress")]
    ToolProgress(SessionToolProgress),

    #[serde(rename = "session.next.tool.success")]
    ToolSuccess(SessionToolSuccess),

    #[serde(rename = "session.next.tool.failed")]
    ToolFailed(SessionToolFailed),

    #[serde(rename = "session.next.retried")]
    Retried(SessionRetried),

    #[serde(rename = "session.next.compaction.started")]
    CompactionStarted(SessionCompactionStarted),

    #[serde(rename = "session.next.compaction.delta")]
    CompactionDelta(SessionCompactionDelta),

    #[serde(rename = "session.next.compaction.ended")]
    CompactionEnded(SessionCompactionEnded),

    #[serde(rename = "session.next.revert.staged")]
    RevertStaged(SessionRevertStaged),

    #[serde(rename = "session.next.revert.cleared")]
    RevertCleared(SessionRevertBase),

    #[serde(rename = "session.next.revert.committed")]
    RevertCommitted(SessionRevertCommitted),

    #[serde(rename = "session.status")]
    Status(SessionStatusEvent),

    #[serde(rename = "session.idle")]
    Idle(SessionIdleEvent),

    #[serde(rename = "session.compacted")]
    Compacted(SessionCompactedEvent),
}

// --- Base event data structs ---

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionBase {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub session_id: SessionID,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionPromptFields {
    #[serde(flatten)]
    pub base: SessionBase,
    pub message_id: SessionMessageID,
    pub prompt: Prompt,
    pub delivery: SessionDelivery,
}

// --- Individual event structs ---

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionAgentSwitched {
    #[serde(flatten)]
    pub base: SessionBase,
    pub message_id: SessionMessageID,
    pub agent: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionModelSwitched {
    #[serde(flatten)]
    pub base: SessionBase,
    pub message_id: SessionMessageID,
    pub model: ModelRef,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionMoved {
    #[serde(flatten)]
    pub base: SessionBase,
    pub location: LocationRef,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subdirectory: Option<RelativePath>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionPrompted {
    #[serde(flatten)]
    pub prompt_fields: SessionPromptFields,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionPromptAdmitted {
    #[serde(flatten)]
    pub prompt_fields: SessionPromptFields,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionContextUpdated {
    #[serde(flatten)]
    pub base: SessionBase,
    pub message_id: SessionMessageID,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionSynthetic {
    #[serde(flatten)]
    pub base: SessionBase,
    pub message_id: SessionMessageID,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionShellStarted {
    #[serde(flatten)]
    pub base: SessionBase,
    pub message_id: SessionMessageID,
    #[serde(rename = "callID")]
    pub call_id: String,
    pub command: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionShellEnded {
    #[serde(flatten)]
    pub base: SessionBase,
    #[serde(rename = "callID")]
    pub call_id: String,
    pub output: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionStepStarted {
    #[serde(flatten)]
    pub base: SessionBase,
    pub assistant_message_id: SessionMessageID,
    pub agent: String,
    pub model: ModelRef,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snapshot: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StepTokens {
    pub input: f64,
    pub output: f64,
    pub reasoning: f64,
    pub cache: StepTokenCache,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StepTokenCache {
    pub read: f64,
    pub write: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionStepEnded {
    #[serde(flatten)]
    pub base: SessionBase,
    pub assistant_message_id: SessionMessageID,
    pub finish: String,
    pub cost: f64,
    pub tokens: StepTokens,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snapshot: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub files: Option<Vec<RelativePath>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionStepFailed {
    #[serde(flatten)]
    pub base: SessionBase,
    pub assistant_message_id: SessionMessageID,
    pub error: SessionMessageUnknownError,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionTextStarted {
    #[serde(flatten)]
    pub base: SessionBase,
    pub assistant_message_id: SessionMessageID,
    pub text_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionTextDelta {
    #[serde(flatten)]
    pub base: SessionBase,
    pub assistant_message_id: SessionMessageID,
    pub text_id: String,
    pub delta: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionTextEnded {
    #[serde(flatten)]
    pub base: SessionBase,
    pub assistant_message_id: SessionMessageID,
    pub text_id: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionReasoningStarted {
    #[serde(flatten)]
    pub base: SessionBase,
    pub assistant_message_id: SessionMessageID,
    pub reasoning_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider_metadata: Option<ProviderMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionReasoningDelta {
    #[serde(flatten)]
    pub base: SessionBase,
    pub assistant_message_id: SessionMessageID,
    pub reasoning_id: String,
    pub delta: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionReasoningEnded {
    #[serde(flatten)]
    pub base: SessionBase,
    pub assistant_message_id: SessionMessageID,
    pub reasoning_id: String,
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider_metadata: Option<ProviderMetadata>,
}

// --- Tool events ---

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionToolInputStarted {
    #[serde(flatten)]
    pub base: SessionBase,
    pub assistant_message_id: SessionMessageID,
    #[serde(rename = "callID")]
    pub call_id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionToolInputDelta {
    #[serde(flatten)]
    pub base: SessionBase,
    pub assistant_message_id: SessionMessageID,
    #[serde(rename = "callID")]
    pub call_id: String,
    pub delta: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionToolInputEnded {
    #[serde(flatten)]
    pub base: SessionBase,
    pub assistant_message_id: SessionMessageID,
    #[serde(rename = "callID")]
    pub call_id: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionToolCalled {
    #[serde(flatten)]
    pub base: SessionBase,
    pub assistant_message_id: SessionMessageID,
    #[serde(rename = "callID")]
    pub call_id: String,
    pub tool: String,
    pub input: HashMap<String, serde_json::Value>,
    pub provider: SessionToolProvider,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionToolProvider {
    pub executed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<ProviderMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionToolProgress {
    #[serde(flatten)]
    pub base: SessionBase,
    pub assistant_message_id: SessionMessageID,
    #[serde(rename = "callID")]
    pub call_id: String,
    pub structured: HashMap<String, serde_json::Value>,
    pub content: Vec<ToolContent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionToolSuccess {
    #[serde(flatten)]
    pub base: SessionBase,
    pub assistant_message_id: SessionMessageID,
    #[serde(rename = "callID")]
    pub call_id: String,
    pub structured: HashMap<String, serde_json::Value>,
    pub content: Vec<ToolContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_paths: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    pub provider: SessionToolProvider,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionToolFailed {
    #[serde(flatten)]
    pub base: SessionBase,
    pub assistant_message_id: SessionMessageID,
    #[serde(rename = "callID")]
    pub call_id: String,
    pub error: SessionMessageUnknownError,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    pub provider: SessionToolProvider,
}

// --- Retry ---

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionRetryError {
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_code: Option<f64>,
    pub is_retryable: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_headers: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_body: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionRetried {
    #[serde(flatten)]
    pub base: SessionBase,
    pub attempt: f64,
    pub error: SessionRetryError,
}

// --- Compaction ---

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionCompactionStarted {
    #[serde(flatten)]
    pub base: SessionBase,
    pub message_id: SessionMessageID,
    pub reason: CompactionReason,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionCompactionDelta {
    #[serde(flatten)]
    pub base: SessionBase,
    pub message_id: SessionMessageID,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionCompactionEnded {
    #[serde(flatten)]
    pub base: SessionBase,
    pub message_id: SessionMessageID,
    pub reason: CompactionReason,
    pub text: String,
    pub recent: String,
}

// --- Revert ---

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionRevertBase {
    #[serde(flatten)]
    pub base: SessionBase,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionRevertStaged {
    #[serde(flatten)]
    pub base: SessionBase,
    pub revert: RevertState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionRevertCommitted {
    #[serde(flatten)]
    pub base: SessionBase,
    pub message_id: SessionMessageID,
}

// --- Status ---

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionStatusEvent {
    #[serde(flatten)]
    pub base: SessionBase,
    pub status: SessionStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionIdleEvent {
    #[serde(flatten)]
    pub base: SessionBase,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionCompactedEvent {
    #[serde(flatten)]
    pub base: SessionBase,
}
