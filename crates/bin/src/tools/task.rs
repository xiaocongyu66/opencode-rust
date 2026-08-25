//! Task system — shared in-memory task store for the Task* tools.
//!
//! Aligned with claude-code-best task model:
//! - Task: { id, subject, description, status, activeForm, metadata, blocks, blockedBy }

use std::collections::HashMap;
use std::sync::LazyLock;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};

/// Task status (matches the TaskStatusSchema enum in claude-code-best).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Pending,
    InProgress,
    Completed,
}

impl TaskStatus {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "pending" => Some(Self::Pending),
            "in_progress" => Some(Self::InProgress),
            "completed" => Some(Self::Completed),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
        }
    }
}

/// A task item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub subject: String,
    pub description: String,
    pub status: TaskStatus,
    pub active_form: String,
    pub metadata: serde_json::Value,
    pub blocks: Vec<String>,
    pub blocked_by: Vec<String>,
}

/// Global in-memory task store keyed by id.
pub static TASK_STORE: LazyLock<Mutex<HashMap<String, Task>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// Counter for generating unique task ids.
static TASK_COUNTER: LazyLock<Mutex<u64>> = LazyLock::new(|| Mutex::new(0));

/// Generate a unique task id like "task_1", "task_2", ...
pub fn next_task_id() -> String {
    let mut c = TASK_COUNTER.lock().unwrap();
    *c += 1;
    format!("task_{}", c)
}

/// List all tasks in insertion order.
pub fn list_tasks() -> Vec<Task> {
    let store = TASK_STORE.lock().unwrap();
    let mut tasks: Vec<Task> = store.values().cloned().collect();
    tasks.sort_by(|a, b| a.id.cmp(&b.id));
    tasks
}

/// Get a task by id.
pub fn get_task(id: &str) -> Option<Task> {
    TASK_STORE.lock().unwrap().get(id).cloned()
}

/// Insert or replace a task.
pub fn put_task(task: Task) {
    TASK_STORE.lock().unwrap().insert(task.id.clone(), task);
}

/// Remove a task by id.
pub fn delete_task(id: &str) -> bool {
    TASK_STORE.lock().unwrap().remove(id).is_some()
}
