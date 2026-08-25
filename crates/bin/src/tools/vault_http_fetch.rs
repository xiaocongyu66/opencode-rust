//! VaultHttpFetch tool — fetch from a vault-protected URL.
use async_trait::async_trait;
use serde::Deserialize;
use crate::tools::tool::{Tool, ToolContext, ToolFailure, ToolResult};

pub struct VaultHttpFetchTool;
impl VaultHttpFetchTool { pub fn new() -> Self { Self } }

#[derive(Deserialize)]
struct VaultHttpFetchInput {
    url: String,
    #[serde(default)]
    prompt: Option<String>,
}

#[async_trait]
impl Tool for VaultHttpFetchTool {
    fn name(&self) -> &str { "VaultHttpFetch" }
    fn description(&self) -> &str { "Fetches content from a vault-protected URL." }
    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "url": { "type": "string", "description": "URL to fetch" },
                "prompt": { "type": "string", "description": "Prompt to apply to content" }
            },
            "required": ["url"]
        })
    }
    async fn execute(&self, params: serde_json::Value, _ctx: &ToolContext) -> Result<ToolResult, ToolFailure> {
        let input: VaultHttpFetchInput = serde_json::from_value(params)?;
        Ok(ToolResult::text(format!("Vault fetch {} (not yet implemented)", input.url)))
    }
}
