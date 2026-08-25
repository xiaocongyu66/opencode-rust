//! MCP tool — call an MCP server tool.
use async_trait::async_trait;
use serde::Deserialize;
use crate::tools::tool::{Tool, ToolContext, ToolFailure, ToolResult};

pub struct McpTool;
impl McpTool { pub fn new() -> Self { Self } }

#[derive(Deserialize)]
struct McpInput {
    #[serde(rename = "mcp_server")]
    mcp_server: String,
    #[serde(rename = "tool_name")]
    tool_name: String,
    #[serde(default)]
    arguments: Option<serde_json::Value>,
}

#[async_trait]
impl Tool for McpTool {
    fn name(&self) -> &str { "MCP" }
    fn description(&self) -> &str {
        "Calls a tool on a connected MCP (Model Context Protocol) server. \
         Pass the server name, tool name, and arguments."
    }
    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "mcp_server": { "type": "string", "description": "Name of the MCP server" },
                "tool_name": { "type": "string", "description": "Name of the tool on the MCP server" },
                "arguments": { "type": "object", "description": "Arguments for the MCP tool" }
            },
            "required": ["mcp_server", "tool_name"]
        })
    }
    async fn execute(&self, params: serde_json::Value, _ctx: &ToolContext) -> Result<ToolResult, ToolFailure> {
        let input: McpInput = serde_json::from_value(params)?;
        Ok(ToolResult::text(format!(
            "MCP call to {}/{} (MCP client not yet implemented)",
            input.mcp_server, input.tool_name
        )))
    }
}
