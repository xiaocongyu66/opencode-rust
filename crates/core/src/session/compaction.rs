//! Session compaction — compresses conversation context.

use opencode_schema::ids::SessionID;

pub struct SessionCompaction;

impl SessionCompaction {
    pub async fn compact(_session_id: &SessionID) -> Result<(), String> {
        todo!("Compaction requires an LLM provider to generate the summary")
    }
}
