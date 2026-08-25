//! Goal tool — set/update the current goal.
use async_trait::async_trait;
use serde::Deserialize;
use crate::tools::tool::{Tool, ToolContext, ToolFailure, ToolResult};

pub struct GoalTool;
impl GoalTool { pub fn new() -> Self { Self } }

#[derive(Deserialize)]
struct GoalInput {
    #[serde(default)]
    goal: Option<String>,
    #[serde(default)]
    action: Option<String>,
}

#[async_trait]
impl Tool for GoalTool {
    fn name(&self) -> &str { "GoalTool" }
    fn description(&self) -> &str { "Sets or updates the current goal for the session." }
    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "goal": { "type": "string", "description": "The goal text" },
                "action": { "type": "string", "description": "Action: set, get, clear" }
            }
        })
    }
    async fn execute(&self, params: serde_json::Value, _ctx: &ToolContext) -> Result<ToolResult, ToolFailure> {
        let input: GoalInput = serde_json::from_value(params)?;
        Ok(ToolResult::text(format!("Goal {:?} (action={:?}, not yet implemented)", input.goal, input.action)))
    }
}
