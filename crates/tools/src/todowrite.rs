//! TodoWrite tool — structured task list management.

use async_trait::async_trait;
use serde::Deserialize;
use crate::tool::{Tool, ToolContext, ToolFailure, ToolResult};

pub struct TodoWriteTool;

impl TodoWriteTool {
    pub fn new() -> Self { Self }
}

#[derive(Deserialize)]
struct TodoItem {
    content: String,
    status: String,
    priority: String,
}

#[derive(Deserialize)]
struct TodoWriteInput {
    todos: Vec<TodoItem>,
}

#[async_trait]
impl Tool for TodoWriteTool {
    fn name(&self) -> &str { "todowrite" }

    fn description(&self) -> &str {
        "Create and maintain a structured task list for the current coding session. Tracks progress and organizes multi-step work."
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
                            "content": { "type": "string", "description": "Brief description of the task" },
                            "status": { "type": "string", "enum": ["pending", "in_progress", "completed", "cancelled"] },
                            "priority": { "type": "string", "enum": ["high", "medium", "low"] }
                        },
                        "required": ["content", "status", "priority"]
                    }
                }
            },
            "required": ["todos"]
        })
    }

    async fn execute(&self, params: serde_json::Value, _ctx: &ToolContext) -> Result<ToolResult, ToolFailure> {
        let input: TodoWriteInput = serde_json::from_value(params)?;
        let summary = input.todos.iter()
            .map(|t| format!("[{}] {} ({}): {}", t.status, t.priority, t.content, t.content))
            .collect::<Vec<_>>()
            .join("\n");
        Ok(ToolResult::text(format!("Updated {} todos:\n{}", input.todos.len(), summary)))
    }
}
