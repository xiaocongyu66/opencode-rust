//! Session status management.
//!
//! Ported from `session/status.ts`.
//! Tracks per-session status (idle/busy/retry) and publishes events on change.

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;

use crate::core::event::EventBus;
use crate::schema::event::EventPayload;
use crate::schema::ids::SessionID;
use crate::schema::session::SessionStatus;

pub struct SessionStatusManager {
    state: Arc<RwLock<HashMap<SessionID, SessionStatus>>>,
    events: Arc<EventBus>,
}

impl SessionStatusManager {
    pub fn new(events: Arc<EventBus>) -> Self {
        Self {
            state: Arc::new(RwLock::new(HashMap::new())),
            events,
        }
    }

    pub async fn get(&self, session_id: &SessionID) -> SessionStatus {
        self.state
            .read()
            .await
            .get(session_id)
            .cloned()
            .unwrap_or(SessionStatus::Idle)
    }

    pub async fn list(&self) -> HashMap<SessionID, SessionStatus> {
        self.state.read().await.clone()
    }

    pub async fn set(&self, session_id: SessionID, status: SessionStatus) {
        let mut data = self.state.write().await;
        let event_type = match &status {
            SessionStatus::Idle => {
                data.remove(&session_id);
                let payload = serde_json::json!({
                    "sessionID": session_id.as_str(),
                    "status": status,
                });
                let event = EventPayload {
                    id: crate::schema::ids::EventID::new(),
                    event_type: "session.status".to_string(),
                    data: payload,
                    durable: None,
                    location: None,
                    metadata: None,
                };
                self.events.publish(event);

                let idle_payload = serde_json::json!({
                    "sessionID": session_id.as_str(),
                });
                let idle_event = EventPayload {
                    id: crate::schema::ids::EventID::new(),
                    event_type: "session.idle".to_string(),
                    data: idle_payload,
                    durable: None,
                    location: None,
                    metadata: None,
                };
                self.events.publish(idle_event);
                return;
            }
            _ => "session.status",
        };

        data.insert(session_id.clone(), status.clone());

        let payload = serde_json::json!({
            "sessionID": session_id.as_str(),
            "status": status,
        });
        let event = EventPayload {
            id: crate::schema::ids::EventID::new(),
            event_type: event_type.to_string(),
            data: payload,
            durable: None,
            location: None,
            metadata: None,
        };
        self.events.publish(event);
    }
}
