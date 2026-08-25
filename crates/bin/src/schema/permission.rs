//! Permission data models.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::schema::ids::{PermissionID, SessionID};

/// The source of a permission request.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum PermissionSource {
    #[serde(rename = "tool")]
    Tool {
        #[serde(rename = "messageID")]
        message_id: String,
        #[serde(rename = "callID")]
        call_id: String,
    },
}

/// A permission request.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionRequest {
    pub id: PermissionID,
    pub session_id: SessionID,
    pub action: String,
    pub resources: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub save: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<PermissionSource>,
}

/// The user's reply to a permission request.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PermissionReply {
    Once,
    Always,
    Reject,
}

/// The effect of a permission rule.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PermissionEffect {
    Allow,
    Deny,
    Ask,
}

/// A single permission rule.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionRule {
    pub action: String,
    pub resource: String,
    pub effect: PermissionEffect,
}

/// A ruleset of permission rules.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PermissionRuleset(pub Vec<PermissionRule>);
