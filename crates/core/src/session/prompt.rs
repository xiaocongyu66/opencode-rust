//! Session prompt — handles prompt admission and delivery.

use opencode_schema::ids::{SessionID, SessionMessageID};
use opencode_schema::prompt::Prompt;
use opencode_schema::session::{SessionDelivery, SessionInputAdmitted};

pub struct SessionPrompt;

impl SessionPrompt {
    pub async fn admit(
        _session_id: SessionID,
        _message_id: SessionMessageID,
        _prompt: Prompt,
        _delivery: SessionDelivery,
    ) -> SessionInputAdmitted {
        todo!("Prompt admission requires a session store and durable event log")
    }
}
