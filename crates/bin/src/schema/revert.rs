//! Revert data models.

use serde::{Deserialize, Serialize};

use crate::schema::common::RelativePath;
use crate::schema::ids::SessionMessageID;

/// A file diff in a revert.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RevertFileDiff {
    pub path: RelativePath,
    pub status: RevertFileDiffStatus,
    pub additions: u64,
    pub deletions: u64,
    pub patch: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RevertFileDiffStatus {
    Added,
    Modified,
    Deleted,
}

/// Revert state.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RevertState {
    #[serde(rename = "messageID")]
    pub message_id: SessionMessageID,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub part_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snapshot: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diff: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub files: Option<Vec<RevertFileDiff>>,
}

/// Standalone file diff info.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileDiffInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub patch: Option<String>,
    pub additions: f64,
    pub deletions: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<RevertFileDiffStatus>,
}
