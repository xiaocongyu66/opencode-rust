//! Prompt data models.

use serde::{Deserialize, Serialize};


/// A text source range.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptSource {
    pub start: f64,
    pub end: f64,
    pub text: String,
}

/// A file attachment in a prompt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileAttachment {
    pub uri: String,
    pub mime: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<PromptSource>,
}

/// An agent attachment in a prompt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentAttachment {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<PromptSource>,
}

/// A user prompt.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Prompt {
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub files: Option<Vec<FileAttachment>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agents: Option<Vec<AgentAttachment>>,
}

/// Prompt input (slightly different shape from Prompt).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PromptInput {
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub files: Option<Vec<PromptInputFileAttachment>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agents: Option<Vec<AgentAttachment>>,
}

/// File attachment in a PromptInput (no mime field).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptInputFileAttachment {
    pub uri: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<PromptSource>,
}

// Re-export RelativePath for downstream use
pub use crate::common::RelativePath as _RelativePath;
