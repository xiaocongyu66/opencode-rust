//! EnterWorktree tool — create and enter a git worktree.
use async_trait::async_trait;
use serde::Deserialize;
use crate::tools::tool::{Tool, ToolContext, ToolFailure, ToolResult};

pub struct EnterWorktreeTool;
impl EnterWorktreeTool { pub fn new() -> Self { Self } }

#[derive(Deserialize)]
struct EnterWorktreeInput {
    #[serde(default)]
    path: Option<String>,
    #[serde(default)]
    branch: Option<String>,
}

#[async_trait]
impl Tool for EnterWorktreeTool {
    fn name(&self) -> &str { "EnterWorktree" }
    fn description(&self) -> &str {
        "Creates a git worktree and switches the session to it. Useful for \
         isolating work on a branch."
    }
    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": { "type": "string", "description": "Path for the new worktree" },
                "branch": { "type": "string", "description": "Branch name for the worktree" }
            }
        })
    }
    async fn execute(&self, params: serde_json::Value, _ctx: &ToolContext) -> Result<ToolResult, ToolFailure> {
        let input: EnterWorktreeInput = serde_json::from_value(params)?;
        Ok(ToolResult::text(format!(
            "Enter worktree (path={:?}, branch={:?}) — git worktree not yet implemented",
            input.path, input.branch
        )))
    }
}
