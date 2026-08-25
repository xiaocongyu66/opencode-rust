//! LocalMemoryRecall tool — recall memories from local memory store.
use async_trait::async_trait;
use serde::Deserialize;
use crate::tools::tool::{Tool, ToolContext, ToolFailure, ToolResult};

pub struct LocalMemoryRecallTool;
impl LocalMemoryRecallTool { pub fn new() -> Self { Self } }

#[derive(Deserialize)]
struct MemoryRecallInput {
    query: String,
    #[serde(default)]
    limit: Option<usize>,
}

#[async_trait]
impl Tool for LocalMemoryRecallTool {
    fn name(&self) -> &str { "LocalMemoryRecall" }
    fn description(&self) -> &str {
        "Searches the local memory store for relevant memories matching the \
         query. Returns up to limit results."
    }
    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "query": { "type": "string", "description": "Search query" },
                "limit": { "type": "number", "description": "Max results (default 5)" }
            },
            "required": ["query"]
        })
    }
    async fn execute(&self, params: serde_json::Value, _ctx: &ToolContext) -> Result<ToolResult, ToolFailure> {
        let input: MemoryRecallInput = serde_json::from_value(params)?;
        Ok(ToolResult::text(format!(
            "Memory recall for '{}' (limit={:?}) — memory store not yet implemented",
            input.query, input.limit
        )))
    }
}
