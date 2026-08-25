//! Agent data models.

use serde::{Deserialize, Serialize};

use crate::ids::{AgentID, ModelID, ProviderID, VariantID};
use crate::permission::PermissionRuleset;

/// Agent color — hex string or named theme color.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AgentColor {
    Named(NamedColor),
    Hex(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NamedColor {
    #[serde(rename = "primary")]
    Primary,
    #[serde(rename = "secondary")]
    Secondary,
    #[serde(rename = "accent")]
    Accent,
    #[serde(rename = "success")]
    Success,
    #[serde(rename = "warning")]
    Warning,
    #[serde(rename = "error")]
    Error,
    #[serde(rename = "info")]
    Info,
}

/// Agent mode.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentMode {
    Subagent,
    Primary,
    All,
}

/// A reference to a model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelRef {
    pub id: ModelID,
    pub provider_id: ProviderID,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variant: Option<VariantID>,
}

/// Provider request shape (headers + body).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProviderRequest {
    pub headers: std::collections::HashMap<String, String>,
    pub body: serde_json::Map<String, serde_json::Value>,
}

/// Agent information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    pub id: AgentID,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<ModelRef>,
    pub request: ProviderRequest,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub mode: AgentMode,
    pub hidden: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<AgentColor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub steps: Option<u64>,
    pub permissions: PermissionRuleset,
}

impl AgentInfo {
    pub fn empty(id: AgentID) -> Self {
        Self {
            id,
            model: None,
            request: ProviderRequest::default(),
            system: None,
            description: None,
            mode: AgentMode::All,
            hidden: false,
            color: None,
            steps: None,
            permissions: PermissionRuleset::default(),
        }
    }
}

// Re-export common types used by other modules
pub use crate::common::AbsolutePath;
