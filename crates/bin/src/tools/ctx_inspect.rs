//! CtxInspect tool — inspect the current context.
use async_trait::async_trait;
use crate::tools::tool::{Tool, ToolContext, ToolFailure, ToolResult};

pub struct CtxInspectTool;
impl CtxInspectTool { pub fn new() -> Self { Self } }

#[async_trait]
impl Tool for CtxInspectTool {
    fn name(&self) -> &str { "CtxInspect" }
    fn description(&self) -> &str { "Inspects the current context (messages, tools, state)." }
    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({"type": "object", "properties": {}})
    }
    async fn execute(&self, _params: serde_json::Value, _ctx: &ToolContext) -> Result<ToolResult, ToolFailure> {
        Ok(ToolResult::text("Context inspection not yet implemented"))
    }
}
