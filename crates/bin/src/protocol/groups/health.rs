//! Health routes — `packages/protocol/src/groups/health.ts`

use axum::Router;

use crate::protocol::api::ApiGroup;

/// `GET` — Check whether the API server is ready to accept requests.
pub const HEALTH_GET: &str = "/api/health";

/// Health API group.
pub struct HealthGroup;

impl ApiGroup for HealthGroup {
    type State = ();
    fn routes() -> Router<()> {
        Router::new()
    }
}
