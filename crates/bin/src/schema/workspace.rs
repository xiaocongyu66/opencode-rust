//! Workspace data models.

use serde::{Deserialize, Serialize};

use crate::schema::ids::WorkspaceID;

/// Workspace info (re-exports WorkspaceID as the primary type).
pub use crate::schema::ids::WorkspaceID as ID;

/// Workspace info struct.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceInfo {
    pub id: WorkspaceID,
}
