//! Session revert management.
//!
//! Ported from `session/revert.ts`.
//! Handles reverting sessions to previous message states via snapshots.

use crate::schema::ids::SessionID;
use crate::schema::session::SessionMessage;

/// Revert input.
#[derive(Debug, Clone)]
pub struct RevertInput {
    pub session_id: SessionID,
    pub message_id: String,
    pub part_id: Option<String>,
}

/// Revert state info.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct RevertInfo {
    pub message_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub part_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snapshot: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diff: Option<String>,
}

/// Revert a session to a given message/part.
pub async fn revert(
    messages: &[SessionMessage],
    input: &RevertInput,
) -> Result<Option<RevertInfo>, String> {
    let mut last_user_id: Option<String> = None;
    let mut rev: Option<RevertInfo> = None;

    for msg in messages {
        if matches!(msg, SessionMessage::User { .. }) {
            last_user_id = Some(match msg {
                SessionMessage::User { id, .. } => id.to_string(),
                _ => String::new(),
            });
        }

        if rev.is_some() {
            continue;
        }

        let msg_id = match msg {
            SessionMessage::User { id, .. } => id.to_string(),
            SessionMessage::Assistant { id, .. } => id.to_string(),
            SessionMessage::Synthetic { id, .. } => id.to_string(),
            _ => continue,
        };

        if msg_id == input.message_id && input.part_id.is_none() {
            rev = Some(RevertInfo {
                message_id: last_user_id.clone().unwrap_or(msg_id),
                part_id: None,
                snapshot: None,
                diff: None,
            });
        }
    }

    Ok(rev)
}
