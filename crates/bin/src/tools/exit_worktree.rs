//! ExitWorktree tool — exit and clean up a git worktree.
use async_trait::async_trait;
use crate::tools::tool::{Tool, ToolContext, ToolFailure, ToolResult};

pub struct ExitWorktreeTool;
impl ExitWorktreeTool { pub fn new() -> Self { Self } }

#[async_trait]
impl Tool for ExitWorktreeTool {
    fn name(&self) -> &str { "ExitWorktree" }
    fn description(&self) -> &str {
        "Exits the current git worktree and cleans it up."
    }
    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({"type": "object", "properties": {}})
    }
    async fn execute(&self, _params: serde_json::Value, _ctx: &ToolContext) -> Result<ToolResult, ToolFailure> {
        Ok(ToolResult::text("Exit worktree — git worktree not yet implemented"))
    }
}
