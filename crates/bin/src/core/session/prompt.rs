//! Session prompt — handles prompt admission and delivery.
//!
//! Ported from `core/src/session/prompt.ts` (re-exports schema types)
//! and the prompt admission logic from `core/src/session.ts`.

use crate::schema::ids::{SessionID, SessionMessageID};
use crate::schema::prompt::Prompt;
use crate::schema::session::{SessionDelivery, SessionInputAdmitted};

/// Delivery mode for a prompt.
pub type Delivery = SessionDelivery;

/// An admitted session input.
pub type InputAdmitted = SessionInputAdmitted;

pub struct SessionPrompt;

impl SessionPrompt {
    /// Admit a prompt into a session's durable inbox.
    ///
    /// In the TS version, this writes a `session_input` row and publishes
    /// a `PromptAdmitted` event. Here we return the admitted record for
    /// the caller to persist.
    pub fn admit(
        session_id: SessionID,
        message_id: SessionMessageID,
        prompt: Prompt,
        delivery: SessionDelivery,
    ) -> SessionInputAdmitted {
        SessionInputAdmitted {
            admitted_seq: 0,
            id: message_id,
            session_id,
            prompt,
            delivery,
            time_created: chrono::Utc::now(),
            promoted_seq: None,
        }
    }

    /// Validate a prompt before admission.
    pub fn validate(prompt: &Prompt) -> Result<(), String> {
        if prompt.text.trim().is_empty() && prompt.files.is_none() && prompt.agents.is_none() {
            return Err("Prompt must have text, files, or agents".to_string());
        }
        Ok(())
    }
}
