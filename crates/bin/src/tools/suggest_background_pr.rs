//! SuggestBackgroundPR tool — suggest a background PR.
use async_trait::async_trait;
use serde::Deserialize;
use crate::tools::tool::{Tool, ToolContext, ToolFailure, ToolResult};

pub struct SuggestBackgroundPrTool;
impl SuggestBackgroundPrTool { pub fn new() -> Self { Self } }

#[derive(Deserialize)]
struct SuggestBackgroundPrInput {
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    body: Option<String>,
}

#[async_trait]
impl Tool for SuggestBackgroundPrTool {
    fn name(&self) -> &str { "SuggestBackgroundPR" }
    fn description(&self) -> &str { "Suggests a background PR with title and body." }
    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "title": { "type": "string", "description": "PR title" },
                "body": { "type": "string", "description": "PR body" }
            }
        })
    }
    async fn execute(&self, params: serde_json::Value, _ctx: &ToolContext) -> Result<ToolResult, ToolFailure> {
        let input: SuggestBackgroundPrInput = serde_json::from_value(params)?;
        Ok(ToolResult::text(format!("Suggest PR {:?} (not yet implemented)", input.title)))
    }
}
