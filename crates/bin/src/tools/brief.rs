//! Brief tool — generate a brief/summary.
use async_trait::async_trait;
use serde::Deserialize;
use crate::tools::tool::{Tool, ToolContext, ToolFailure, ToolResult};

pub struct BriefTool;
impl BriefTool { pub fn new() -> Self { Self } }

#[derive(Deserialize)]
struct BriefInput {
    #[serde(default)]
    topic: Option<String>,
    #[serde(default)]
    detail: Option<String>,
}

#[async_trait]
impl Tool for BriefTool {
    fn name(&self) -> &str { "Brief" }
    fn description(&self) -> &str { "Generates a brief/summary on a topic." }
    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "topic": { "type": "string", "description": "Topic to brief" },
                "detail": { "type": "string", "description": "Detail level: short, medium, long" }
            }
        })
    }
    async fn execute(&self, params: serde_json::Value, _ctx: &ToolContext) -> Result<ToolResult, ToolFailure> {
        let input: BriefInput = serde_json::from_value(params)?;
        Ok(ToolResult::text(format!("Brief on {:?} (not yet implemented)", input.topic)))
    }
}
