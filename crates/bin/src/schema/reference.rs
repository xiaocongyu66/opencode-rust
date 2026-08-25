//! Reference data models.

use serde::{Deserialize, Serialize};

use crate::schema::common::AbsolutePath;

/// Reference source — local or git.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ReferenceSource {
    #[serde(rename = "local")]
    Local {
        path: AbsolutePath,
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        hidden: Option<bool>,
    },
    #[serde(rename = "git")]
    Git {
        repository: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        branch: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        hidden: Option<bool>,
    },
}

/// Reference info.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferenceInfo {
    pub name: String,
    pub path: AbsolutePath,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hidden: Option<bool>,
    pub source: ReferenceSource,
}
