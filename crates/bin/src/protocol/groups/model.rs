//! Model routes — `packages/protocol/src/groups/model.ts`

use axum::Router;

use crate::protocol::api::ApiGroup;

/// `GET` — List available models ordered by release date.
pub const MODEL_LIST: &str = "/api/model";

/// Model API group.
pub struct ModelGroup;

impl ApiGroup for ModelGroup {
    type State = ();
    fn routes() -> Router<()> {
        Router::new()
    }
}
