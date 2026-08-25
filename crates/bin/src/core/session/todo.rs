//! Session todo management.
//!
//! Ported from `session/todo.ts`.
//! Manages per-session todo lists.

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;

use crate::core::event::EventBus;
use crate::schema::ids::SessionID;

/// Todo item.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Todo {
    pub content: String,
    pub status: TodoStatus,
    pub priority: TodoPriority,
}

/// Todo status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TodoStatus {
    Pending,
    InProgress,
    Completed,
    Cancelled,
}

impl Default for TodoStatus {
    fn default() -> Self {
        Self::Pending
    }
}

/// Todo priority.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TodoPriority {
    Low,
    Medium,
    High,
}

impl Default for TodoPriority {
    fn default() -> Self {
        Self::Medium
    }
}

/// Session todo manager.
pub struct SessionTodoManager {
    todos: Arc<RwLock<HashMap<SessionID, Vec<Todo>>>>,
    events: Arc<EventBus>,
}

impl SessionTodoManager {
    pub fn new(events: Arc<EventBus>) -> Self {
        Self {
            todos: Arc::new(RwLock::new(HashMap::new())),
            events,
        }
    }

    pub async fn get(&self, session_id: &SessionID) -> Vec<Todo> {
        self.todos
            .read()
            .await
            .get(session_id)
            .cloned()
            .unwrap_or_default()
    }

    pub async fn update(&self, session_id: SessionID, todos: Vec<Todo>) {
        self.todos.write().await.insert(session_id.clone(), todos.clone());

        let payload = serde_json::json!({
            "sessionID": session_id.as_str(),
            "todos": todos,
        });
        let event = crate::schema::event::EventPayload {
            id: crate::schema::ids::EventID::new(),
            event_type: "session.todo.updated".to_string(),
            data: payload,
            durable: None,
            location: None,
            metadata: None,
        };
        self.events.publish(event);
    }
}
