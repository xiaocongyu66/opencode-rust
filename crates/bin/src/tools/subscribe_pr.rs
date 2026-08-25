//! SubscribePR tool — subscribe to a PR's updates.
use async_trait::async_trait;
use serde::Deserialize;
use crate::tools::tool::{Tool, ToolContext, ToolFailure, ToolResult};

pub struct SubscribePrTool;
impl SubscribePrTool { pub fn new() -> Self { Self } }

#[derive(Deserialize)]
struct SubscribePrInput {
    #[serde(default)]
    repo: Option<String>,
    #[serde(default)]
    pr: Option<u64>,
}

#[async_trait]
impl Tool for SubscribePrTool {
    fn name(&self) -> &str { "SubscribePR" }
    fn description(&self) -> &str { "Subscribes to updates on a GitHub PR." }
    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "repo": { "type": "string", "description": "owner/repo" },
                "pr": { "type": "number", "description": "PR number" }
            }
        })
    }
    async fn execute(&self, params: serde_json::Value, _ctx: &ToolContext) -> Result<ToolResult, ToolFailure> {
        let input: SubscribePrInput = serde_json::from_value(params)?;
        Ok(ToolResult::text(format!("Subscribe PR {:?}# {:?} (not yet implemented)", input.repo, input.pr)))
    }
}
