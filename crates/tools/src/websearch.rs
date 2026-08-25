//! WebSearch tool — search the web.

use async_trait::async_trait;
use serde::Deserialize;
use crate::tool::{Tool, ToolContext, ToolFailure, ToolResult};

pub struct WebSearchTool;

impl WebSearchTool {
    pub fn new() -> Self { Self }
}

#[derive(Deserialize)]
struct WebSearchInput {
    query: String,
    #[serde(default)]
    count: Option<usize>,
}

#[async_trait]
impl Tool for WebSearchTool {
    fn name(&self) -> &str { "websearch" }

    fn description(&self) -> &str {
        "Search the web for real-time information. Returns content from the most relevant websites."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "query": { "type": "string", "description": "Search query" },
                "count": { "type": "integer", "description": "Number of results to return" }
            },
            "required": ["query"]
        })
    }

    async fn execute(&self, params: serde_json::Value, _ctx: &ToolContext) -> Result<ToolResult, ToolFailure> {
        let _input: WebSearchInput = serde_json::from_value(params)?;
        Err(ToolFailure::Message("Web search requires a configured search provider".to_string()))
    }
}
