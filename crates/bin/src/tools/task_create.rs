//! TaskCreate tool — create a new task.
//!
//! Aligned with claude-code-best TaskCreateTool:
//! - `subject` (required): brief title
//! - `description` (required): what needs to be done
//! - `activeForm` (optional): present-continuous form for spinner
//! - `metadata` (optional): arbitrary metadata

use async_trait::async_trait;
use serde::Deserialize;

use crate::tools::task::{next_task_id, put_task, Task, TaskStatus};
use crate::tools::tool::{Tool, ToolContext, ToolFailure, ToolResult};

pub struct TaskCreateTool;

impl TaskCreateTool {
    pub fn new() -> Self {
        Self
    }
}

#[derive(Deserialize)]
struct TaskCreateInput {
    subject: String,
    description: String,
    #[serde(default, rename = "activeForm")]
    active_form: Option<String>,
    #[serde(default)]
    metadata: Option<serde_json::Value>,
}

#[async_trait]
impl Tool for TaskCreateTool {
    fn name(&self) -> &str { "TaskCreate"
    }

    fn description(&self) -> &str {
        "Creates a new task with a subject, description, and optional metadata. \
         Returns the task id."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "subject": { "type": "string", "description": "A brief title for the task" },
                "description": { "type": "string", "description": "What needs to be done" },
                "activeForm": { "type": "string", "description": "Present continuous form shown in spinner when in_progress" },
                "metadata": { "type": "object", "description": "Arbitrary metadata to attach to the task" }
            },
            "required": ["subject", "description"]
        })
    }

    async fn execute(
        &self,
        params: serde_json::Value,
        _ctx: &ToolContext,
    ) -> Result<ToolResult, ToolFailure> {
        let input: TaskCreateInput = serde_json::from_value(params)?;
        let id = next_task_id();
        let task = Task {
            id: id.clone(),
            subject: input.subject,
            description: input.description,
            status: TaskStatus::Pending,
            active_form: input.active_form.unwrap_or_default(),
            metadata: input.metadata.unwrap_or(serde_json::Value::Null),
            blocks: Vec::new(),
            blocked_by: Vec::new(),
        };
        put_task(task);
        Ok(ToolResult::text(format!("Created task {}", id)))
    }
}
