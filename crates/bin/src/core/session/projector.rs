//! Session projector — projects durable events into visible messages.
//!
//! Ported from `core/src/session/projector.ts`.
//! The TS version subscribes to EventV2 and projects each event into
//! SQL tables (session, session_message, etc). Here we provide a
//! trait-based projector that applies events to a SessionStore.

use std::sync::Arc;
use async_trait::async_trait;

use crate::schema::ids::SessionID;
use crate::schema::session::SessionMessage;
use crate::schema::session_event::SessionEvent;

/// Event projection trait — applies events to produce visible messages.
#[async_trait]
pub trait EventProjector: Send + Sync {
    async fn project(&self, event: &SessionEvent) -> Result<(), String>;
}

/// Session projector — projects events into session messages.
pub struct SessionProjector {
    store: Arc<dyn crate::core::session::SessionStore>,
}

impl SessionProjector {
    pub fn new(store: Arc<dyn crate::core::session::SessionStore>) -> Self {
        Self { store }
    }

    /// Apply a session event to the store.
    pub async fn apply_event(&self, event: &SessionEvent) -> Result<(), String> {
        match event {
            SessionEvent::AgentSwitched(e) => {
                let msg = SessionMessage::AgentSwitched {
                    id: e.message_id.clone(),
                    metadata: None,
                    time: crate::schema::session::MessageTime { created: e.base.timestamp },
                    agent: e.agent.clone(),
                };
                self.store.append_message(&e.base.session_id, msg).await;
                Ok(())
            }
            SessionEvent::ModelSwitched(e) => {
                let msg = SessionMessage::ModelSwitched {
                    id: e.message_id.clone(),
                    metadata: None,
                    time: crate::schema::session::MessageTime { created: e.base.timestamp },
                    model: e.model.clone(),
                };
                self.store.append_message(&e.base.session_id, msg).await;
                Ok(())
            }
            SessionEvent::ContextUpdated(e) => {
                let msg = SessionMessage::System {
                    id: e.message_id.clone(),
                    metadata: None,
                    time: crate::schema::session::MessageTime { created: e.base.timestamp },
                    text: e.text.clone(),
                };
                self.store.append_message(&e.base.session_id, msg).await;
                Ok(())
            }
            SessionEvent::Synthetic(e) => {
                let msg = SessionMessage::Synthetic {
                    id: e.message_id.clone(),
                    metadata: None,
                    time: crate::schema::session::MessageTime { created: e.base.timestamp },
                    session_id: e.base.session_id.clone(),
                    text: e.text.clone(),
                };
                self.store.append_message(&e.base.session_id, msg).await;
                Ok(())
            }
            SessionEvent::ShellStarted(e) => {
                let msg = SessionMessage::Shell {
                    id: e.message_id.clone(),
                    metadata: None,
                    time: crate::schema::session::ShellTime {
                        created: e.base.timestamp,
                        completed: None,
                    },
                    call_id: e.call_id.clone(),
                    command: e.command.clone(),
                    output: String::new(),
                };
                self.store.append_message(&e.base.session_id, msg).await;
                Ok(())
            }
            SessionEvent::ShellEnded(e) => {
                let msg = SessionMessage::Shell {
                    id: crate::schema::ids::SessionMessageID::new(),
                    metadata: None,
                    time: crate::schema::session::ShellTime {
                        created: e.base.timestamp,
                        completed: Some(e.base.timestamp),
                    },
                    call_id: e.call_id.clone(),
                    command: String::new(),
                    output: e.output.clone(),
                };
                self.store.append_message(&e.base.session_id, msg).await;
                Ok(())
            }
            SessionEvent::StepStarted(e) => {
                let msg = SessionMessage::Assistant {
                    id: e.assistant_message_id.clone(),
                    metadata: None,
                    time: crate::schema::session::AssistantTime {
                        created: e.base.timestamp,
                        completed: None,
                    },
                    agent: e.agent.clone(),
                    model: e.model.clone(),
                    content: vec![],
                    snapshot: e.snapshot.as_ref().map(|s| crate::schema::session::AssistantSnapshot {
                        start: Some(s.clone()),
                        end: None,
                        files: None,
                    }),
                    finish: None,
                    cost: None,
                    tokens: None,
                    error: None,
                };
                self.store.append_message(&e.base.session_id, msg).await;
                Ok(())
            }
            SessionEvent::StepEnded(e) => {
                tracing::info!("Session {} step ended: {}", e.base.session_id, e.finish);
                Ok(())
            }
            SessionEvent::StepFailed(e) => {
                tracing::warn!("Session {} step failed: {}", e.base.session_id, e.error.message);
                Ok(())
            }
            SessionEvent::TextStarted(e) => {
                tracing::debug!("Session {} text started: {}", e.base.session_id, e.text_id);
                Ok(())
            }
            SessionEvent::TextDelta(e) => {
                tracing::trace!("Session {} text delta: {}", e.base.session_id, e.text_id);
                Ok(())
            }
            SessionEvent::TextEnded(e) => {
                tracing::debug!("Session {} text ended: {}", e.base.session_id, e.text_id);
                Ok(())
            }
            SessionEvent::ToolInputStarted(e) => {
                tracing::debug!("Session {} tool input started: {}", e.base.session_id, e.call_id);
                Ok(())
            }
            SessionEvent::ToolInputDelta(e) => {
                tracing::trace!("Session {} tool input delta: {}", e.base.session_id, e.call_id);
                Ok(())
            }
            SessionEvent::ToolInputEnded(e) => {
                tracing::debug!("Session {} tool input ended: {}", e.base.session_id, e.call_id);
                Ok(())
            }
            SessionEvent::ToolCalled(e) => {
                tracing::info!("Session {} tool called: {}", e.base.session_id, e.tool);
                Ok(())
            }
            SessionEvent::ToolProgress(e) => {
                tracing::debug!("Session {} tool progress: {}", e.base.session_id, e.call_id);
                Ok(())
            }
            SessionEvent::ToolSuccess(e) => {
                tracing::info!("Session {} tool success: {}", e.base.session_id, e.call_id);
                Ok(())
            }
            SessionEvent::ToolFailed(e) => {
                tracing::warn!("Session {} tool failed: {}", e.base.session_id, e.call_id);
                Ok(())
            }
            SessionEvent::ReasoningStarted(e) => {
                tracing::debug!("Session {} reasoning started: {}", e.base.session_id, e.reasoning_id);
                Ok(())
            }
            SessionEvent::ReasoningDelta(e) => {
                tracing::trace!("Session {} reasoning delta: {}", e.base.session_id, e.reasoning_id);
                Ok(())
            }
            SessionEvent::ReasoningEnded(e) => {
                tracing::debug!("Session {} reasoning ended: {}", e.base.session_id, e.reasoning_id);
                Ok(())
            }
            SessionEvent::CompactionEnded(e) => {
                let msg = SessionMessage::Compaction {
                    id: e.message_id.clone(),
                    metadata: None,
                    time: crate::schema::session::MessageTime { created: e.base.timestamp },
                    reason: e.reason.clone(),
                    summary: e.text.clone(),
                    recent: e.recent.clone(),
                };
                self.store.append_message(&e.base.session_id, msg).await;
                Ok(())
            }
            SessionEvent::Prompted(e) => {
                let msg = SessionMessage::User {
                    id: e.prompt_fields.message_id.clone(),
                    metadata: None,
                    time: crate::schema::session::MessageTime { created: e.prompt_fields.base.timestamp },
                    text: e.prompt_fields.prompt.text.clone(),
                    files: e.prompt_fields.prompt.files.clone(),
                    agents: e.prompt_fields.prompt.agents.clone(),
                };
                self.store.append_message(&e.prompt_fields.base.session_id, msg).await;
                Ok(())
            }
            SessionEvent::CompactionStarted(e) => {
                tracing::info!("Session {} compaction started: {:?}", e.base.session_id, e.reason);
                Ok(())
            }
            _ => Ok(()),
        }
    }

    /// Project all events for a session into visible messages.
    pub async fn project_all(&self, _session_id: &SessionID, events: &[SessionEvent]) -> Result<(), String> {
        for event in events {
            self.apply_event(event).await?;
        }
        Ok(())
    }
}

#[async_trait]
impl EventProjector for SessionProjector {
    async fn project(&self, event: &SessionEvent) -> Result<(), String> {
        self.apply_event(event).await
    }
}
