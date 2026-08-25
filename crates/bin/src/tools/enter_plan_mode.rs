//! EnterPlanMode tool — enter plan mode (read-only analysis).
//!
//! Aligned with claude-code-best EnterPlanModeTool.

use async_trait::async_trait;
use crate::tools::tool::{Tool, ToolContext, ToolFailure, ToolResult};

pub struct EnterPlanModeTool;

impl EnterPlanModeTool {
    pub fn new() -> Self { Self }
}

#[async_trait]
impl Tool for EnterPlanModeTool {
    fn name(&self) -> &str { "EnterPlanMode" }
    fn description(&self) -> &str {
        "Enters plan mode, a read-only mode where the assistant analyzes the \
         codebase and proposes a plan before making changes. No tools that \
         modify files can be used in this mode."
    }
    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({"type": "object", "properties": {}})
    }
    async fn execute(&self, _params: serde_json::Value, _ctx: &ToolContext) -> Result<ToolResult, ToolFailure> {
        Ok(ToolResult::text("Entered plan mode (read-only analysis)."))
    }
}
