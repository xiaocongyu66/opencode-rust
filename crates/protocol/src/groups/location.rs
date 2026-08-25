//! Location routes — `packages/protocol/src/groups/location.ts`
//!
//! The location group exposes a single endpoint to resolve the requested or
//! default location. It also defines the shared `LocationQuery` structure
//! used by many other groups.

use axum::Router;
use serde::{Deserialize, Serialize};

use crate::api::ApiGroup;

/// `GET` — Resolve the requested location or the server default location.
pub const LOCATION_GET: &str = "/api/location";

/// Query parameters for location-scoped endpoints.
///
/// Serialized as a deep-object query parameter (e.g.
/// `?location[directory]=...&location[workspace]=...`).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LocationQuery {
    /// The location to resolve. When omitted, the server default is used.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location: Option<LocationRef>,
}

/// A reference to a workspace location.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LocationRef {
    /// Absolute path to the working directory.
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "directory")]
    pub directory: Option<String>,
    /// Workspace identifier within the directory.
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "workspace")]
    pub workspace: Option<String>,
}

/// Location API group.
pub struct LocationGroup;

impl ApiGroup for LocationGroup {
    type State = ();
    fn routes() -> Router<()> {
        Router::new()
    }
}
