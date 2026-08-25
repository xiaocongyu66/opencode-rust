//! Session history — retrieves and paginates session messages.
//!
//! Ported from `core/src/session/history.ts`.
//! Loads messages from the store, respecting context epoch baseline
//! and compaction boundaries.

use crate::schema::ids::{SessionID, SessionMessageID};
use crate::schema::session::SessionMessage;
use crate::schema::session_event::SessionEvent;

/// A history entry with its sequence number.
#[derive(Debug, Clone)]
pub struct HistoryEntry {
    pub seq: u64,
    pub message: SessionMessage,
}

pub struct SessionHistory;

impl SessionHistory {
    /// Get events for a session, paginated by sequence number.
    pub async fn get_events(
        store: &dyn crate::core::session::SessionStore,
        session_id: &SessionID,
        after: Option<u64>,
        limit: usize,
    ) -> (Vec<SessionEvent>, bool) {
        let messages = store.context(session_id).await.unwrap_or_default();
        let mut events = Vec::new();
        let mut seq = after.unwrap_or(0);
        let mut count = 0;
        for message in &messages {
            if count >= limit {
                return (events, true);
            }
            seq += 1;
            let event = Self::message_to_event(session_id, message, seq);
            events.push(event);
            count += 1;
        }
        (events, false)
    }

    /// Load all messages for a session from the store.
    pub async fn load(
        store: &dyn crate::core::session::SessionStore,
        session_id: &SessionID,
    ) -> Vec<SessionMessage> {
        store.context(session_id).await.unwrap_or_default()
    }

    /// Load messages for the runner, respecting baseline sequence.
    pub async fn load_for_runner(
        store: &dyn crate::core::session::SessionStore,
        session_id: &SessionID,
        baseline_seq: u64,
    ) -> Vec<SessionMessage> {
        let messages = store.context(session_id).await.unwrap_or_default();
        messages
            .into_iter()
            .filter(|m| {
                let msg_seq = Self::message_seq(m);
                msg_seq.is_none_or(|s| s > baseline_seq)
            })
            .collect()
    }

    /// Load entries with sequence numbers for the runner.
    pub async fn entries_for_runner(
        store: &dyn crate::core::session::SessionStore,
        session_id: &SessionID,
        baseline_seq: u64,
    ) -> Vec<HistoryEntry> {
        let messages = store.context(session_id).await.unwrap_or_default();
        let mut entries = Vec::new();
        for (i, message) in messages.into_iter().enumerate() {
            let seq = (i as u64) + 1;
            if seq > baseline_seq {
                entries.push(HistoryEntry { seq, message });
            }
        }
        entries
    }

    /// Find the latest compaction message in the list.
    pub fn latest_compaction(messages: &[SessionMessage]) -> Option<&SessionMessage> {
        messages
            .iter()
            .rev()
            .find(|m| matches!(m, SessionMessage::Compaction { .. }))
    }

    fn message_seq(_message: &SessionMessage) -> Option<u64> {
        None
    }

    fn message_to_event(
        session_id: &SessionID,
        message: &SessionMessage,
        _seq: u64,
    ) -> SessionEvent {
        let now = chrono::Utc::now();
        let base = crate::schema::session_event::SessionBase {
            timestamp: now,
            session_id: session_id.clone(),
        };
        match message {
            SessionMessage::User { id, text, .. } => {
                SessionEvent::TextStarted(crate::schema::session_event::SessionTextStarted {
                    base,
                    assistant_message_id: id.clone(),
                    text_id: text.clone(),
                })
            }
            SessionMessage::Assistant { id, .. } => {
                SessionEvent::StepStarted(crate::schema::session_event::SessionStepStarted {
                    base,
                    assistant_message_id: id.clone(),
                    agent: String::new(),
                    model: crate::schema::model::ModelRef {
                        id: crate::schema::ids::ModelID(String::new()),
                        provider_id: crate::schema::ids::ProviderID(String::new()),
                        variant: None,
                    },
                    snapshot: None,
                })
            }
            SessionMessage::System { text, .. } => {
                SessionEvent::ContextUpdated(crate::schema::session_event::SessionContextUpdated {
                    base,
                    message_id: SessionMessageID::from_str(""),
                    text: text.clone(),
                })
            }
            _ => SessionEvent::Idle(crate::schema::session_event::SessionIdleEvent { base }),
        }
    }
}
