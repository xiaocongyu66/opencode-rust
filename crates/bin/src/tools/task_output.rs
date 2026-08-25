//! TaskOutput tool — get output from a background task.
//!
//! Aligned with claude-code-best TaskOutputTool:
//! - `task_id` (required): the task ID to get output from
//! - `block` (optional, default true): whether to wait for completion
//! - `timeout` (optional, default 30000): max wait time in ms

use async_trait::async_trait;
use serde::Deserialize;

use crate::tools::tool::{Tool, ToolContext, ToolFailure, ToolResult};

pub struct TaskOutputTool;

impl TaskOutputTool {
    pub fn new() -> Self {
        Self
    }
}

#[derive(Deserialize)]
struct TaskOutputInput {
    #[serde(rename = "task_id")]
    task_id: String,
    #[serde(default)]
    block: Option<bool>,
    #[serde(default)]
    timeout: Option<u64>,
}

#[async_trait]
impl Tool for TaskOutputTool {
    fn name(&self) -> &str { "TaskOutput"
    }

    fn description(&self) -> &str {
        "Gets the output of a background task. If block is true (default), \
         waits up to timeout ms for the task to complete."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "task_id": { "type": "string", "description": "The task ID to get output from" },
                "block": { "type": "boolean", "description": "Whether to wait for completion (default true)" },
                "timeout": { "type": "number", "description": "Max wait time in ms (default 30000, max 600000)", "minimum": 0, "maximum": 600000 }
            },
            "required": ["task_id"]
        })
    }

    async fn execute(
        &self,
        params: serde_json::Value,
        _ctx: &ToolContext,
    ) -> Result<ToolResult, ToolFailure> {
        let input: TaskOutputInput = serde_json::from_value(params)?;
        let _block = input.block.unwrap_or(true);
        let _timeout = input.timeout.unwrap_or(30_000).min(600_000);
        // Background task output streaming not yet implemented.
        Ok(ToolResult::text(format!(
            "Output for task {} not available (background task management not yet implemented)",
            input.task_id
        )))
    }
}
