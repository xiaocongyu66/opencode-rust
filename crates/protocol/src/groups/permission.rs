//! Permission routes — `packages/protocol/src/groups/permission.ts`
//!
//! Permission endpoints are split between location-scoped routes (request
//! list, saved list, saved remove) and session-scoped routes (create, list,
//! get, reply) that require session placement middleware.

use axum::Router;

use crate::api::ApiGroup;

// ---------------------------------------------------------------------------
// Location-scoped routes
// ---------------------------------------------------------------------------

/// `GET` — List pending permission requests for a location.
pub const PERMISSION_REQUEST_LIST: &str = "/api/permission/request";
/// `GET` — List saved permissions, optionally filtered by project.
pub const PERMISSION_SAVED_LIST: &str = "/api/permission/saved";
/// `DELETE` — Remove a saved permission by ID.
pub const PERMISSION_SAVED_REMOVE: &str = "/api/permission/saved/:id";

// ---------------------------------------------------------------------------
// Session-scoped routes
// ---------------------------------------------------------------------------

/// `POST` — Create (evaluate) a permission request for a session.
pub const SESSION_PERMISSION_CREATE: &str = "/api/session/:sessionID/permission";
/// `GET` — List pending permission requests owned by a session.
pub const SESSION_PERMISSION_LIST: &str = "/api/session/:sessionID/permission";
/// `GET` — Get a pending permission request owned by a session.
pub const SESSION_PERMISSION_GET: &str = "/api/session/:sessionID/permission/:requestID";
/// `POST` — Reply to a pending permission request.
pub const SESSION_PERMISSION_REPLY: &str = "/api/session/:sessionID/permission/:requestID/reply";

/// Permission API group.
pub struct PermissionGroup;

impl ApiGroup for PermissionGroup {
    type State = ();
    fn routes() -> Router<()> {
        Router::new()
    }
}
