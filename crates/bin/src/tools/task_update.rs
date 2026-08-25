//! TaskUpdate tool — update an existing task.
//!
//! Aligned with claude-code-best TaskUpdateTool:
//! - `taskId` (required): the task id
//! - `subject`, `description`, `activeForm`, `status` (all optional)
//! - `status` can be "deleted" to remove the task
//! - `addBlocks`, `addBlockedBy` (optional arrays of task ids)

use async_trait::async_trait;
use serde::Deserialize;

use crate::tools::task::{delete_task, get_task, put_task, TaskStatus};
use crate::tools::tool::{Tool, ToolContext, ToolFailure, ToolResult};

pub struct TaskUpdateTool;

impl TaskUpdateTool {
    pub fn new() -> Self {
        Self
    }
}

#[derive(Deserialize)]
struct TaskUpdateInput {
    #[serde(rename = "taskId")]
    task_id: String,
    #[serde(default)]
    subject: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default, rename = "activeForm")]
    active_form: Option<String>,
    #[serde(default)]
    status: Option<String>,
    #[serde(default, rename = "addBlocks")]
    add_blocks: Option<Vec<String>>,
    #[serde(default, rename = "addBlockedBy")]
    add_blocked_by: Option<Vec<String>>,
}

#[async_trait]
impl Tool for TaskUpdateTool {
    fn name(&self) -> &str { "TaskUpdate"
    }

    fn description(&self) -> &str {
        "Updates a task's subject, description, activeForm, or status. Use \
         status 'deleted' to remove a task. Supports adding blocking \
         relationships via addBlocks/addBlockedBy."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "taskId": { "type": "string", "description": "The ID of the task to update" },
                "subject": { "type": "string", "description": "New subject for the task" },
                "description": { "type": "string", "description": "New description for the task" },
                "activeForm": { "type": "string", "description": "Present continuous form shown in spinner" },
                "status": { "type": "string", "enum": ["pending", "in_progress", "completed", "deleted"], "description": "New status for the task" },
                "addBlocks": { "type": "array", "items": { "type": "string" }, "description": "Task IDs that this task blocks" },
                "addBlockedBy": { "type": "array", "items": { "type": "string" }, "description": "Task IDs that block this task" }
            },
            "required": ["taskId"]
        })
    }

    async fn execute(
        &self,
        params: serde_json::Value,
        _ctx: &ToolContext,
    ) -> Result<ToolResult, ToolFailure> {
        let input: TaskUpdateInput = serde_json::from_value(params)?;

        // Handle deletion via status="deleted".
        if input.status.as_deref() == Some("deleted") {
            if delete_task(&input.task_id) {
                return Ok(ToolResult::text(format!("Deleted task {}", input.task_id)));
            }
            return Ok(ToolResult::text(format!("Task {} not found", input.task_id)));
        }

        let mut task = get_task(&input.task_id)
            .ok_or_else(|| ToolFailure::Message(format!("Task {} not found", input.task_id)))?;

        if let Some(s) = input.subject {
            task.subject = s;
        }
        if let Some(d) = input.description {
            task.description = d;
        }
        if let Some(a) = input.active_form {
            task.active_form = a;
        }
        if let Some(status) = input.status {
            if let Some(s) = TaskStatus::from_str(&status) {
                task.status = s;
            }
        }
        if let Some(blocks) = input.add_blocks {
            for b in blocks {
                if !task.blocks.contains(&b) {
                    task.blocks.push(b);
                }
            }
        }
        if let Some(blocked_by) = input.add_blocked_by {
            for b in blocked_by {
                if !task.blocked_by.contains(&b) {
                    task.blocked_by.push(b);
                }
            }
        }

        let summary = format!(
            "Updated task {}: {} [{}]",
            task.id,
            task.subject,
            task.status.as_str()
        );
        put_task(task);
        Ok(ToolResult::text(summary))
    }
}
