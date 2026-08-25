//! Location data models.

use serde::{Deserialize, Serialize};

use crate::common::AbsolutePath;
use crate::ids::{ProjectID, WorkspaceID};

/// A reference to a location (directory + optional workspace).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationRef {
    pub directory: AbsolutePath,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_id: Option<WorkspaceID>,
}

/// Full location info.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationInfo {
    pub directory: AbsolutePath,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_id: Option<WorkspaceID>,
    pub project: LocationProject,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationProject {
    pub id: ProjectID,
    pub directory: AbsolutePath,
}

/// A location response wrapper.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationResponse<T> {
    pub location: LocationInfo,
    pub data: T,
}
