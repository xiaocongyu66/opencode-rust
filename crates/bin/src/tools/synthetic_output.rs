//! SyntheticOutput tool — internal tool for deferred tool output.
use async_trait::async_trait;
use serde::Deserialize;
use crate::tools::tool::{Tool, ToolContext, ToolFailure, ToolResult};

pub struct SyntheticOutputTool;
impl SyntheticOutputTool { pub fn new() -> Self { Self } }

#[derive(Deserialize)]
struct SyntheticOutputInput {
    #[serde(default)]
    output: Option<String>,
}

#[async_trait]
impl Tool for SyntheticOutputTool {
    fn name(&self) -> &str { "StructuredOutput" }
    fn description(&self) -> &str { "Internal tool: synthetic output for deferred tool loading." }
    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "output": { "type": "string", "description": "Synthetic output text" }
            }
        })
    }
    async fn execute(&self, params: serde_json::Value, _ctx: &ToolContext) -> Result<ToolResult, ToolFailure> {
        let input: SyntheticOutputInput = serde_json::from_value(params)?;
        Ok(ToolResult::text(input.output.unwrap_or_default()))
    }
}
