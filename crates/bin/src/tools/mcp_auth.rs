//! McpAuth tool — authenticate with an MCP server.
use async_trait::async_trait;
use serde::Deserialize;
use crate::tools::tool::{Tool, ToolContext, ToolFailure, ToolResult};

pub struct McpAuthTool;
impl McpAuthTool { pub fn new() -> Self { Self } }

#[derive(Deserialize)]
struct McpAuthInput {
    #[serde(rename = "mcp_server")]
    mcp_server: String,
    #[serde(default)]
    action: Option<String>,
}

#[async_trait]
impl Tool for McpAuthTool {
    fn name(&self) -> &str { "McpAuth" }
    fn description(&self) -> &str {
        "Authenticates with an MCP server that requires OAuth or API key. \
         Supports 'start' and 'status' actions."
    }
    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "mcp_server": { "type": "string", "description": "Name of the MCP server" },
                "action": { "type": "string", "description": "Auth action: start, status" }
            },
            "required": ["mcp_server"]
        })
    }
    async fn execute(&self, params: serde_json::Value, _ctx: &ToolContext) -> Result<ToolResult, ToolFailure> {
        let input: McpAuthInput = serde_json::from_value(params)?;
        Ok(ToolResult::text(format!(
            "MCP auth for {} (action={:?}) not yet implemented",
            input.mcp_server, input.action
        )))
    }
}
