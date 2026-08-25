//! Todo data models.

use serde::{Deserialize, Serialize};

use crate::schema::ids::SessionID;

/// Todo info.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoInfo {
    pub content: String,
    pub status: String,
    pub priority: String,
}

/// Todo updated event data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoUpdated {
    pub session_id: SessionID,
    pub todos: Vec<TodoInfo>,
}
