//! ReadMcpResource tool — read a resource from an MCP server.
use async_trait::async_trait;
use serde::Deserialize;
use crate::tools::tool::{Tool, ToolContext, ToolFailure, ToolResult};

pub struct ReadMcpResourceTool;
impl ReadMcpResourceTool { pub fn new() -> Self { Self } }

#[derive(Deserialize)]
struct ReadMcpResourceInput {
    #[serde(rename = "mcp_server")]
    mcp_server: String,
    #[serde(rename = "resource_uri")]
    resource_uri: String,
}

#[async_trait]
impl Tool for ReadMcpResourceTool {
    fn name(&self) -> &str { "ReadMcpResource" }
    fn description(&self) -> &str {
        "Reads a resource by URI from a connected MCP server."
    }
    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "mcp_server": { "type": "string", "description": "Name of the MCP server" },
                "resource_uri": { "type": "string", "description": "URI of the resource to read" }
            },
            "required": ["mcp_server", "resource_uri"]
        })
    }
    async fn execute(&self, params: serde_json::Value, _ctx: &ToolContext) -> Result<ToolResult, ToolFailure> {
        let input: ReadMcpResourceInput = serde_json::from_value(params)?;
        Ok(ToolResult::text(format!(
            "Read {} from {} (MCP client not yet implemented)",
            input.resource_uri, input.mcp_server
        )))
    }
}
