//! REPL tool — evaluate code in a REPL.
use async_trait::async_trait;
use serde::Deserialize;
use crate::tools::tool::{Tool, ToolContext, ToolFailure, ToolResult};

pub struct ReplTool;
impl ReplTool { pub fn new() -> Self { Self } }

#[derive(Deserialize)]
struct ReplInput {
    #[serde(default)]
    language: Option<String>,
    #[serde(default)]
    code: Option<String>,
}

#[async_trait]
impl Tool for ReplTool {
    fn name(&self) -> &str { "REPL" }
    fn description(&self) -> &str { "Evaluates code in a REPL (e.g. python, node)." }
    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "language": { "type": "string", "description": "Language: python, node, etc." },
                "code": { "type": "string", "description": "Code to evaluate" }
            }
        })
    }
    async fn execute(&self, params: serde_json::Value, _ctx: &ToolContext) -> Result<ToolResult, ToolFailure> {
        let input: ReplInput = serde_json::from_value(params)?;
        Ok(ToolResult::text(format!("REPL {:?} (not yet implemented)", input.language)))
    }
}
