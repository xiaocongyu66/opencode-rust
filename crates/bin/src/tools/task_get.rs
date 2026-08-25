//! TaskGet tool — retrieve a single task by id.
//!
//! Aligned with claude-code-best TaskGetTool:
//! - `taskId` (required): the ID of the task to retrieve

use async_trait::async_trait;
use serde::Deserialize;

use crate::tools::task::get_task;
use crate::tools::tool::{Tool, ToolContext, ToolFailure, ToolResult};

pub struct TaskGetTool;

impl TaskGetTool {
    pub fn new() -> Self {
        Self
    }
}

#[derive(Deserialize)]
struct TaskGetInput {
    #[serde(rename = "taskId")]
    task_id: String,
}

#[async_trait]
impl Tool for TaskGetTool {
    fn name(&self) -> &str { "TaskGet"
    }

    fn description(&self) -> &str {
        "Retrieves a task by its id, including subject, description, status, \
         blocks, and blockedBy."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "taskId": { "type": "string", "description": "The ID of the task to retrieve" }
            },
            "required": ["taskId"]
        })
    }

    async fn execute(
        &self,
        params: serde_json::Value,
        _ctx: &ToolContext,
    ) -> Result<ToolResult, ToolFailure> {
        let input: TaskGetInput = serde_json::from_value(params)?;
        match get_task(&input.task_id) {
            Some(t) => {
                let out = format!(
                    "Task {}:\n  subject: {}\n  description: {}\n  status: {}\n  activeForm: {}\n  blocks: {}\n  blockedBy: {}",
                    t.id,
                    t.subject,
                    t.description,
                    t.status.as_str(),
                    t.active_form,
                    t.blocks.join(", "),
                    t.blocked_by.join(", ")
                );
                Ok(ToolResult::text(out))
            }
            None => Ok(ToolResult::text(format!("Task {} not found", input.task_id))),
        }
    }
}
