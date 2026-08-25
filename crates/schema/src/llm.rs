//! LLM-related data models.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Provider metadata (arbitrary key-value pairs per provider).
pub type ProviderMetadata = HashMap<String, HashMap<String, serde_json::Value>>;

/// Text content in a tool result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolTextContent {
    pub text: String,
}

/// File content in a tool result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolFileContent {
    pub uri: String,
    pub mime: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// Tagged union of tool content.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ToolContent {
    #[serde(rename = "text")]
    Text(ToolTextContent),
    #[serde(rename = "file")]
    File(ToolFileContent),
}
