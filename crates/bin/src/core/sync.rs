//! Sync module — event ID generation for cross-instance synchronization.
//!
//! Ported from `sync/schema.ts`.

use crate::schema::common::ascending;

/// Event ID — branded string prefixed with "evt_".
pub use crate::schema::ids::EventID;

/// Generate a new ascending EventID.
pub fn new_event_id() -> EventID {
    EventID(format!("evt_{}", ascending()))
}
