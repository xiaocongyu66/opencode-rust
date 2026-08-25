//! TerminalCapture tool — capture the terminal screen.
use async_trait::async_trait;
use crate::tools::tool::{Tool, ToolContext, ToolFailure, ToolResult};

pub struct TerminalCaptureTool;
impl TerminalCaptureTool { pub fn new() -> Self { Self } }

#[async_trait]
impl Tool for TerminalCaptureTool {
    fn name(&self) -> &str { "TerminalCapture" }
    fn description(&self) -> &str { "Captures the current terminal screen as text." }
    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({"type": "object", "properties": {}})
    }
    async fn execute(&self, _params: serde_json::Value, _ctx: &ToolContext) -> Result<ToolResult, ToolFailure> {
        Ok(ToolResult::text("Terminal capture not yet implemented"))
    }
}
