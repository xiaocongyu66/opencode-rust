//! Session data models.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::common::RelativePath;
use crate::ids::{AgentID, ProjectID, SessionID, SessionMessageID};
use crate::location::LocationRef;
use crate::llm::ProviderMetadata;
use crate::model::ModelRef;
use crate::prompt::{FileAttachment, Prompt};
use crate::revert::RevertState;
use crate::llm::ToolContent;

/// Session token counts.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionTokens {
    pub input: f64,
    pub output: f64,
    pub reasoning: f64,
    pub cache: SessionTokenCache,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionTokenCache {
    pub read: f64,
    pub write: f64,
}

/// Session time info.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionTime {
    pub created: chrono::DateTime<chrono::Utc>,
    pub updated: chrono::DateTime<chrono::Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub archived: Option<chrono::DateTime<chrono::Utc>>,
}

/// Session info.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionInfo {
    pub id: SessionID,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<SessionID>,
    pub project_id: ProjectID,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent: Option<AgentID>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<ModelRef>,
    pub cost: f64,
    pub tokens: SessionTokens,
    pub time: SessionTime,
    pub title: String,
    pub location: LocationRef,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subpath: Option<RelativePath>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revert: Option<RevertState>,
}

/// List anchor for pagination.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionListAnchor {
    pub id: SessionID,
    pub time: f64,
    pub direction: SessionListDirection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionListDirection {
    Previous,
    Next,
}

// ============================================================================
// Session Message
// ============================================================================

/// Session message error.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub struct SessionMessageUnknownError {
    pub message: String,
}

// --- Tool states ---

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "camelCase")]
pub enum ToolState {
    Pending { input: String },
    Running {
        input: HashMap<String, serde_json::Value>,
        structured: HashMap<String, serde_json::Value>,
        content: Vec<ToolContent>,
    },
    Completed {
        input: HashMap<String, serde_json::Value>,
        #[serde(skip_serializing_if = "Option::is_none")]
        attachments: Option<Vec<FileAttachment>>,
        content: Vec<ToolContent>,
        #[serde(skip_serializing_if = "Option::is_none")]
        output_paths: Option<Vec<String>>,
        structured: HashMap<String, serde_json::Value>,
        #[serde(skip_serializing_if = "Option::is_none")]
        result: Option<serde_json::Value>,
    },
    Error {
        input: HashMap<String, serde_json::Value>,
        content: Vec<ToolContent>,
        structured: HashMap<String, serde_json::Value>,
        error: SessionMessageUnknownError,
        #[serde(skip_serializing_if = "Option::is_none")]
        result: Option<serde_json::Value>,
    },
}

// --- Assistant content ---

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum AssistantContent {
    #[serde(rename = "text")]
    Text {
        id: String,
        text: String,
    },
    #[serde(rename = "reasoning")]
    Reasoning {
        id: String,
        text: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        provider_metadata: Option<ProviderMetadata>,
        #[serde(skip_serializing_if = "Option::is_none")]
        time: Option<AssistantReasoningTime>,
    },
    #[serde(rename = "tool")]
    Tool {
        id: String,
        name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        provider: Option<AssistantToolProvider>,
        state: ToolState,
        time: AssistantToolTime,
    },
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssistantReasoningTime {
    pub created: chrono::DateTime<chrono::Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssistantToolProvider {
    pub executed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<ProviderMetadata>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result_metadata: Option<ProviderMetadata>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssistantToolTime {
    pub created: chrono::DateTime<chrono::Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ran: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pruned: Option<chrono::DateTime<chrono::Utc>>,
}

/// Assistant message snapshot.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssistantSnapshot {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub files: Option<Vec<RelativePath>>,
}

/// Assistant message token info.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssistantTokens {
    pub input: f64,
    pub output: f64,
    pub reasoning: f64,
    pub cache: AssistantTokenCache,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssistantTokenCache {
    pub read: f64,
    pub write: f64,
}

/// Base message time.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageTime {
    pub created: chrono::DateTime<chrono::Utc>,
}

/// Shell message time.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShellTime {
    pub created: chrono::DateTime<chrono::Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed: Option<chrono::DateTime<chrono::Utc>>,
}

/// Assistant message time.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssistantTime {
    pub created: chrono::DateTime<chrono::Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed: Option<chrono::DateTime<chrono::Utc>>,
}

// --- Session Message tagged union ---

/// A session message — tagged union of all message types.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum SessionMessage {
    #[serde(rename = "agent-switched")]
    AgentSwitched {
        id: SessionMessageID,
        #[serde(skip_serializing_if = "Option::is_none")]
        metadata: Option<HashMap<String, serde_json::Value>>,
        time: MessageTime,
        agent: String,
    },
    #[serde(rename = "model-switched")]
    ModelSwitched {
        id: SessionMessageID,
        #[serde(skip_serializing_if = "Option::is_none")]
        metadata: Option<HashMap<String, serde_json::Value>>,
        time: MessageTime,
        model: ModelRef,
    },
    #[serde(rename = "user")]
    User {
        id: SessionMessageID,
        #[serde(skip_serializing_if = "Option::is_none")]
        metadata: Option<HashMap<String, serde_json::Value>>,
        time: MessageTime,
        text: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        files: Option<Vec<FileAttachment>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        agents: Option<Vec<crate::prompt::AgentAttachment>>,
    },
    #[serde(rename = "synthetic")]
    Synthetic {
        id: SessionMessageID,
        #[serde(skip_serializing_if = "Option::is_none")]
        metadata: Option<HashMap<String, serde_json::Value>>,
        time: MessageTime,
        session_id: SessionID,
        text: String,
    },
    #[serde(rename = "system")]
    System {
        id: SessionMessageID,
        #[serde(skip_serializing_if = "Option::is_none")]
        metadata: Option<HashMap<String, serde_json::Value>>,
        time: MessageTime,
        text: String,
    },
    #[serde(rename = "shell")]
    Shell {
        id: SessionMessageID,
        #[serde(skip_serializing_if = "Option::is_none")]
        metadata: Option<HashMap<String, serde_json::Value>>,
        time: ShellTime,
        call_id: String,
        command: String,
        output: String,
    },
    #[serde(rename = "assistant")]
    Assistant {
        id: SessionMessageID,
        #[serde(skip_serializing_if = "Option::is_none")]
        metadata: Option<HashMap<String, serde_json::Value>>,
        time: AssistantTime,
        agent: String,
        model: ModelRef,
        content: Vec<AssistantContent>,
        #[serde(skip_serializing_if = "Option::is_none")]
        snapshot: Option<AssistantSnapshot>,
        #[serde(skip_serializing_if = "Option::is_none")]
        finish: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        cost: Option<f64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        tokens: Option<AssistantTokens>,
        #[serde(skip_serializing_if = "Option::is_none")]
        error: Option<SessionMessageUnknownError>,
    },
    #[serde(rename = "compaction")]
    Compaction {
        id: SessionMessageID,
        #[serde(skip_serializing_if = "Option::is_none")]
        metadata: Option<HashMap<String, serde_json::Value>>,
        time: MessageTime,
        reason: CompactionReason,
        summary: String,
        recent: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompactionReason {
    Auto,
    Manual,
}

// ============================================================================
// Session Input
// ============================================================================

/// Session delivery mode.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionDelivery {
    Steer,
    Queue,
}

/// An admitted session input.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionInputAdmitted {
    pub admitted_seq: u64,
    pub id: SessionMessageID,
    pub session_id: SessionID,
    pub prompt: Prompt,
    pub delivery: SessionDelivery,
    pub time_created: chrono::DateTime<chrono::Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub promoted_seq: Option<u64>,
}

// ============================================================================
// Session Status
// ============================================================================

/// Session status info.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum SessionStatus {
    Idle,
    Busy,
    Retry {
        attempt: u64,
        message: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        action: Option<SessionRetryAction>,
        next: u64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionRetryAction {
    pub reason: String,
    pub provider: String,
    pub title: String,
    pub message: String,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub link: Option<String>,
}

// Re-export commonly used types
pub use SessionMessage as Message;
