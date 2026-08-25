//! MCP server configuration.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpTimeout {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub startup: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum McpServer {
    #[serde(rename = "local")]
    Local {
        command: Vec<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        cwd: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        environment: Option<HashMap<String, String>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        disabled: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        timeout: Option<McpTimeout>,
    },
    #[serde(rename = "remote")]
    Remote {
        url: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        disabled: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        timeout: Option<McpTimeout>,
    },
}

pub type McpConfig = HashMap<String, McpServer>;
