//! SearchExtraTools tool — search for deferred tools.
use async_trait::async_trait;
use serde::Deserialize;
use crate::tools::tool::{Tool, ToolContext, ToolFailure, ToolResult};

pub struct SearchExtraToolsTool;
impl SearchExtraToolsTool { pub fn new() -> Self { Self } }

#[derive(Deserialize)]
struct SearchExtraToolsInput {
    query: String,
}

#[async_trait]
impl Tool for SearchExtraToolsTool {
    fn name(&self) -> &str { "SearchExtraTools" }
    fn description(&self) -> &str { "Searches for deferred tools by keyword (TF-IDF index)." }
    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "query": { "type": "string", "description": "Search query" }
            },
            "required": ["query"]
        })
    }
    async fn execute(&self, params: serde_json::Value, _ctx: &ToolContext) -> Result<ToolResult, ToolFailure> {
        let input: SearchExtraToolsInput = serde_json::from_value(params)?;
        Ok(ToolResult::text(format!("Search extra tools '{}' (no deferred tools registered)", input.query)))
    }
}
