//! OverflowTest tool — internal test tool for output overflow handling.
use async_trait::async_trait;
use serde::Deserialize;
use crate::tools::tool::{Tool, ToolContext, ToolFailure, ToolResult};

pub struct OverflowTestTool;
impl OverflowTestTool { pub fn new() -> Self { Self } }

#[derive(Deserialize)]
struct OverflowTestInput {
    #[serde(default)]
    size: Option<usize>,
}

#[async_trait]
impl Tool for OverflowTestTool {
    fn name(&self) -> &str { "OverflowTest" }
    fn description(&self) -> &str { "Internal test tool: generates large output to test overflow handling." }
    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "size": { "type": "number", "description": "Size of output in chars" }
            }
        })
    }
    async fn execute(&self, params: serde_json::Value, _ctx: &ToolContext) -> Result<ToolResult, ToolFailure> {
        let input: OverflowTestInput = serde_json::from_value(params)?;
        let size = input.size.unwrap_or(1000);
        Ok(ToolResult::text("x".repeat(size)))
    }
}
