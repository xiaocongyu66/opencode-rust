//! Project data models.

use serde::{Deserialize, Serialize};

use crate::ids::ProjectID;

/// Project VCS type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProjectVcs {
    #[serde(rename = "git")]
    Git,
}

/// Project icon.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProjectIcon {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub override_: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
}

/// Project commands.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProjectCommands {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start: Option<String>,
}

/// Project time info.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProjectTime {
    pub created: u64,
    pub updated: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initialized: Option<u64>,
}

/// Project info.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectInfo {
    pub id: ProjectID,
    pub worktree: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vcs: Option<ProjectVcs>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<ProjectIcon>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commands: Option<ProjectCommands>,
    pub time: ProjectTime,
    pub sandboxes: Vec<String>,
}
