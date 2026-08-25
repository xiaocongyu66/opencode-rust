//! ListPeers tool — list connected peers.
use async_trait::async_trait;
use crate::tools::tool::{Tool, ToolContext, ToolFailure, ToolResult};

pub struct ListPeersTool;
impl ListPeersTool { pub fn new() -> Self { Self } }

#[async_trait]
impl Tool for ListPeersTool {
    fn name(&self) -> &str { "ListPeers" }
    fn description(&self) -> &str { "Lists connected peer agents." }
    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({"type": "object", "properties": {}})
    }
    async fn execute(&self, _params: serde_json::Value, _ctx: &ToolContext) -> Result<ToolResult, ToolFailure> {
        Ok(ToolResult::text("No peers connected."))
    }
}
