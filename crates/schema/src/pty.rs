//! PTY data models.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::ids::PtyID;

/// PTY info.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PtyInfo {
    pub id: PtyID,
    pub title: String,
    pub command: String,
    pub args: Vec<String>,
    pub cwd: String,
    pub status: PtyStatus,
    pub pid: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PtyStatus {
    Running,
    Exited,
}

/// PTY create input.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PtyCreateInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<HashMap<String, String>>,
}

/// PTY update input.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PtyUpdateInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<PtySize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PtySize {
    pub rows: u64,
    pub cols: u64,
}

/// PTY ticket.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PtyConnectToken {
    pub ticket: String,
    pub expires_in: u64,
}
