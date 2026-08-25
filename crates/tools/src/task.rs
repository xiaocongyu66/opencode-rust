//! Task tool — launch a subagent for complex multi-step tasks.

use async_trait::async_trait;
use serde::Deserialize;
use crate::tool::{Tool, ToolContext, ToolFailure, ToolResult};

pub struct TaskTool;

impl TaskTool {
    pub fn new() -> Self { Self }
}

#[derive(Deserialize)]
struct TaskInput {
    description: String,
    prompt: String,
    subagent_type: String,
}

#[async_trait]
impl Tool for TaskTool {
    fn name(&self) -> &str { "task" }

    fn description(&self) -> &str {
        "Launch a new agent to handle complex, multistep tasks autonomously."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "description": { "type": "string", "description": "A short (3-5 words) description of the task" },
                "prompt": { "type": "string", "description": "The task for the agent to perform" },
                "subagent_type": { "type": "string", "description": "The type of subagent to use" }
            },
            "required": ["description", "prompt", "subagent_type"]
        })
    }

    async fn execute(&self, params: serde_json::Value, _ctx: &ToolContext) -> Result<ToolResult, ToolFailure> {
        let _input: TaskInput = serde_json::from_value(params)?;
        Err(ToolFailure::Message("Subagent execution not yet implemented".to_string()))
    }
}
