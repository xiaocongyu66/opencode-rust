//! Session history — retrieves and paginates session events.

use opencode_schema::ids::SessionID;
use opencode_schema::session_event::SessionEvent;

pub struct SessionHistory;

impl SessionHistory {
    pub async fn get_events(
        _session_id: &SessionID,
        _after: Option<u64>,
        _limit: usize,
    ) -> (Vec<SessionEvent>, bool) {
        todo!("Session history retrieval requires a database connection")
    }
}
