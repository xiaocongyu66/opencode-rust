//! Message routes — `packages/protocol/src/groups/message.ts`

use axum::Router;

use crate::protocol::api::ApiGroup;

/// `GET` — List projected messages for a session.
pub const SESSION_MESSAGES: &str = "/api/session/:sessionID/message";

/// Message API group.
pub struct MessageGroup;

impl ApiGroup for MessageGroup {
    type State = ();
    fn routes() -> Router<()> {
        Router::new()
    }
}
