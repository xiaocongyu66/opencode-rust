//! TaskList tool — list all tasks.
//!
//! Aligned with claude-code-best TaskListTool (empty input schema).

use async_trait::async_trait;

use crate::tools::task::list_tasks;
use crate::tools::tool::{Tool, ToolContext, ToolFailure, ToolResult};

pub struct TaskListTool;

impl TaskListTool {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for TaskListTool {
    fn name(&self) -> &str { "TaskList"
    }

    fn description(&self) -> &str {
        "Lists all tasks with their id, subject, status, and blocking \
         relationships."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {}
        })
    }

    async fn execute(
        &self,
        _params: serde_json::Value,
        _ctx: &ToolContext,
    ) -> Result<ToolResult, ToolFailure> {
        let tasks = list_tasks();
        if tasks.is_empty() {
            return Ok(ToolResult::text("No tasks."));
        }
        let mut out = format!("{} task(s):\n", tasks.len());
        for t in &tasks {
            out.push_str(&format!(
                "  {} [{}] {} (blocks: {}, blocked by: {})\n",
                t.id,
                t.status.as_str(),
                t.subject,
                t.blocks.len(),
                t.blocked_by.len()
            ));
        }
        Ok(ToolResult::text(out))
    }
}
