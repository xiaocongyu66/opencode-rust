//! Tungsten tool — internal/tooling tool.
use async_trait::async_trait;
use serde::Deserialize;
use crate::tools::tool::{Tool, ToolContext, ToolFailure, ToolResult};

pub struct TungstenTool;
impl TungstenTool { pub fn new() -> Self { Self } }

#[derive(Deserialize)]
struct TungstenInput {
    #[serde(default)]
    action: Option<String>,
}

#[async_trait]
impl Tool for TungstenTool {
    fn name(&self) -> &str { "TungstenTool" }
    fn description(&self) -> &str { "Internal tooling tool (Tungsten)." }
    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "action": { "type": "string", "description": "Action to perform" }
            }
        })
    }
    async fn execute(&self, params: serde_json::Value, _ctx: &ToolContext) -> Result<ToolResult, ToolFailure> {
        let input: TungstenInput = serde_json::from_value(params)?;
        Ok(ToolResult::text(format!("Tungsten {:?} (not yet implemented)", input.action)))
    }
}
