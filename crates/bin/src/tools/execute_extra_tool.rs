//! ExecuteExtraTool tool — execute a deferred tool.
use async_trait::async_trait;
use serde::Deserialize;
use crate::tools::tool::{Tool, ToolContext, ToolFailure, ToolResult};

pub struct ExecuteExtraTool;
impl ExecuteExtraTool { pub fn new() -> Self { Self } }

#[derive(Deserialize)]
struct ExecuteExtraToolInput {
    #[serde(default)]
    tool_name: Option<String>,
    #[serde(default)]
    input: Option<serde_json::Value>,
}

#[async_trait]
impl Tool for ExecuteExtraTool {
    fn name(&self) -> &str { "ExecuteExtraTool" }
    fn description(&self) -> &str { "Executes a deferred tool by name with input." }
    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "tool_name": { "type": "string", "description": "Deferred tool name" },
                "input": { "type": "object", "description": "Input for the tool" }
            }
        })
    }
    async fn execute(&self, params: serde_json::Value, _ctx: &ToolContext) -> Result<ToolResult, ToolFailure> {
        let input: ExecuteExtraToolInput = serde_json::from_value(params)?;
        Ok(ToolResult::text(format!("Execute extra tool {:?} (no deferred tools registered)", input.tool_name)))
    }
}
