//! TaskStop tool — stop a background task.
//!
//! Aligned with claude-code-best TaskStopTool:
//! - `task_id` (optional): the ID of the background task to stop
//! - `shell_id` (optional, deprecated): legacy alias for task_id

use async_trait::async_trait;
use serde::Deserialize;

use crate::tools::tool::{Tool, ToolContext, ToolFailure, ToolResult};

pub struct TaskStopTool;

impl TaskStopTool {
    pub fn new() -> Self {
        Self
    }
}

#[derive(Deserialize)]
struct TaskStopInput {
    #[serde(default, rename = "task_id")]
    task_id: Option<String>,
    #[serde(default, rename = "shell_id")]
    shell_id: Option<String>,
}

#[async_trait]
impl Tool for TaskStopTool {
    fn name(&self) -> &str { "TaskStop"
    }

    fn description(&self) -> &str {
        "Stops a background task by id. Accepts task_id (preferred) or \
         shell_id (deprecated alias)."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "task_id": { "type": "string", "description": "The ID of the background task to stop" },
                "shell_id": { "type": "string", "description": "Deprecated: use task_id instead" }
            }
        })
    }

    async fn execute(
        &self,
        params: serde_json::Value,
        _ctx: &ToolContext,
    ) -> Result<ToolResult, ToolFailure> {
        let input: TaskStopInput = serde_json::from_value(params)?;
        let id = input.task_id.or(input.shell_id);
        match id {
            Some(id) => {
                // In-memory task store doesn't track running background tasks
                // yet; for now, just acknowledge the stop request.
                Ok(ToolResult::text(format!(
                    "Stop requested for task {} (background task management not yet implemented)",
                    id
                )))
            }
            None => Ok(ToolResult::text(
                "No task_id provided — nothing to stop".to_string(),
            )),
        }
    }
}
