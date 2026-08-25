//! Execute tool — execute a generic command with arguments.
use async_trait::async_trait;
use serde::Deserialize;
use crate::tools::tool::{Tool, ToolContext, ToolFailure, ToolResult};

pub struct ExecuteTool;
impl ExecuteTool { pub fn new() -> Self { Self } }

#[derive(Deserialize)]
struct ExecuteInput {
    #[serde(default)]
    command: Option<String>,
    #[serde(default)]
    args: Option<Vec<String>>,
}

#[async_trait]
impl Tool for ExecuteTool {
    fn name(&self) -> &str { "ExecuteExtraTool" }
    fn description(&self) -> &str { "Executes a command with arguments (generic)." }
    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "command": { "type": "string", "description": "Command to execute" },
                "args": { "type": "array", "items": { "type": "string" }, "description": "Arguments" }
            }
        })
    }
    async fn execute(&self, params: serde_json::Value, _ctx: &ToolContext) -> Result<ToolResult, ToolFailure> {
        let input: ExecuteInput = serde_json::from_value(params)?;
        Ok(ToolResult::text(format!("Execute {:?} {:?} (not yet implemented)", input.command, input.args)))
    }
}
