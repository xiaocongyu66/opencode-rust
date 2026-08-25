//! Session projector — projects durable events into visible messages.

use opencode_schema::ids::SessionID;
use opencode_schema::session::SessionMessage;

pub struct SessionProjector;

impl SessionProjector {
    pub async fn project(_session_id: &SessionID) -> Vec<SessionMessage> {
        todo!("Projection requires a durable event log")
    }
}
