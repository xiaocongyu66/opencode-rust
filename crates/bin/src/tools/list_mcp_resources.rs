//! ListMcpResources tool — list resources from MCP servers.
use async_trait::async_trait;
use crate::tools::tool::{Tool, ToolContext, ToolFailure, ToolResult};

pub struct ListMcpResourcesTool;
impl ListMcpResourcesTool { pub fn new() -> Self { Self } }

#[async_trait]
impl Tool for ListMcpResourcesTool {
    fn name(&self) -> &str { "ListMcpResourcesTool" }
    fn description(&self) -> &str {
        "Lists resources available on connected MCP servers."
    }
    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({"type": "object", "properties": {}})
    }
    async fn execute(&self, _params: serde_json::Value, _ctx: &ToolContext) -> Result<ToolResult, ToolFailure> {
        Ok(ToolResult::text("No MCP servers connected."))
    }
}
