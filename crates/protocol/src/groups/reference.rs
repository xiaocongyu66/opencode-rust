//! Reference routes — `packages/protocol/src/groups/reference.ts`

use axum::Router;

use crate::api::ApiGroup;

/// `GET` — List references available in the requested location.
pub const REFERENCE_LIST: &str = "/api/reference";

/// Reference API group.
pub struct ReferenceGroup;

impl ApiGroup for ReferenceGroup {
    type State = ();
    fn routes() -> Router<()> {
        Router::new()
    }
}
