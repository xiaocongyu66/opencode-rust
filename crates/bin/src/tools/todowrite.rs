//! TodoWrite tool — manage the session task checklist.
//!
//! Aligned with claude-code-best TodoWriteTool:
//! - `todos` (required): the updated todo list, each item has:
//!   - `content` (string): what to do
//!   - `status` ("pending" | "in_progress" | "completed")
//!   - `activeForm` (string): present-tense form for "currently doing X"

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::tools::tool::{Tool, ToolContext, ToolFailure, ToolResult};

pub struct TodoWriteTool;

impl TodoWriteTool {
    pub fn new() -> Self {
        Self
    }
}

#[derive(Deserialize, Serialize, Clone)]
pub struct TodoItem {
    pub content: String,
    pub status: String,
    #[serde(rename = "activeForm")]
    pub active_form: String,
}

#[derive(Deserialize)]
struct TodoWriteInput {
    todos: Vec<TodoItem>,
}

/// Global session todo list (shared across tool calls within a session).
/// In a full implementation this would live in App state; for now we use a
/// static mutex-guarded list keyed by session id.
pub static TODO_LIST: std::sync::LazyLock<std::sync::Mutex<Vec<TodoItem>>> =
    std::sync::LazyLock::new(|| std::sync::Mutex::new(Vec::new()));

#[async_trait]
impl Tool for TodoWriteTool {
    fn name(&self) -> &str { "TodoWrite"
    }

    fn description(&self) -> &str {
        "Updates the session todo list. Use this to plan and track work: \
         create a list of tasks at the start, mark tasks in_progress when \
         working on them, and completed when done. Each todo needs content, \
         status (pending/in_progress/completed), and activeForm."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "todos": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "content": {
                                "type": "string",
                                "description": "What to do (imperative form)",
                                "minLength": 1
                            },
                            "status": {
                                "type": "string",
                                "enum": ["pending", "in_progress", "completed"],
                                "description": "Current status of the task"
                            },
                            "activeForm": {
                                "type": "string",
                                "description": "Present-continuous form of the content (e.g. 'Updating config file')",
                                "minLength": 1
                            }
                        },
                        "required": ["content", "status", "activeForm"]
                    },
                    "description": "The updated todo list"
                }
            },
            "required": ["todos"]
        })
    }

    async fn execute(
        &self,
        params: serde_json::Value,
        _ctx: &ToolContext,
    ) -> Result<ToolResult, ToolFailure> {
        let input: TodoWriteInput = serde_json::from_value(params)?;

        // Validate statuses
        for todo in &input.todos {
            if !["pending", "in_progress", "completed"].contains(&todo.status.as_str()) {
                return Ok(ToolResult::text(format!(
                    "Invalid status '{}' — must be pending, in_progress, or completed",
                    todo.status
                )));
            }
            if todo.content.is_empty() || todo.active_form.is_empty() {
                return Ok(ToolResult::text(
                    "Each todo needs non-empty content and activeForm".to_string(),
                ));
            }
        }

        let mut list = TODO_LIST.lock().unwrap();
        let old_count = list.len();
        *list = input.todos.clone();
        let new_count = list.len();

        let pending: usize = list.iter().filter(|t| t.status == "pending").count();
        let in_progress: usize = list
            .iter()
            .filter(|t| t.status == "in_progress")
            .count();
        let completed: usize = list.iter().filter(|t| t.status == "completed").count();

        let mut result = format!(
            "Todo list updated ({} → {} items): {} pending, {} in_progress, {} completed\n",
            old_count, new_count, pending, in_progress, completed
        );
        for (i, todo) in list.iter().enumerate() {
            let icon = match todo.status.as_str() {
                "completed" => "[x]",
                "in_progress" => "[>]",
                _ => "[ ]",
            };
            result.push_str(&format!("  {} {}. {}\n", icon, i + 1, todo.content));
        }

        Ok(ToolResult::text(result))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn todo_item_deserializes() {
        let json = serde_json::json!({
            "content": "Write tests",
            "status": "in_progress",
            "activeForm": "Writing tests"
        });
        let item: TodoItem = serde_json::from_value(json).unwrap();
        assert_eq!(item.content, "Write tests");
        assert_eq!(item.status, "in_progress");
        assert_eq!(item.active_form, "Writing tests");
    }

    #[test]
    fn todo_item_requires_active_form() {
        let json = serde_json::json!({
            "content": "Write tests",
            "status": "in_progress"
        });
        let result: Result<TodoItem, _> = serde_json::from_value(json);
        assert!(result.is_err());
    }
}
