//! Event routes — `packages/protocol/src/groups/event.ts`
//!
//! The event group exposes a single SSE streaming endpoint that replays
//! durable events and continues with new events as they are committed.

use axum::Router;

use crate::protocol::api::ApiGroup;

/// `GET` — Subscribe to native event payloads (SSE stream).
pub const EVENT_SUBSCRIBE: &str = "/api/event";

/// Event API group.
pub struct EventGroup;

impl ApiGroup for EventGroup {
    type State = ();
    fn routes() -> Router<()> {
        Router::new()
    }
}
