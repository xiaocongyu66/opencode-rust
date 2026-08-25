//! ExitPlanMode tool — exit plan mode.
//!
//! Aligned with claude-code-best ExitPlanModeV2Tool.

use async_trait::async_trait;
use serde::Deserialize;
use crate::tools::tool::{Tool, ToolContext, ToolFailure, ToolResult};

pub struct ExitPlanModeTool;

impl ExitPlanModeTool {
    pub fn new() -> Self { Self }
}

#[derive(Deserialize)]
struct ExitPlanInput {
    #[serde(default)]
    plan: Option<String>,
}

#[async_trait]
impl Tool for ExitPlanModeTool {
    fn name(&self) -> &str { "ExitPlanMode" }
    fn description(&self) -> &str {
        "Exits plan mode. Optionally takes a plan to present to the user \
         for approval before proceeding with implementation."
    }
    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "plan": { "type": "string", "description": "The plan to present to the user for approval." }
            }
        })
    }
    async fn execute(&self, params: serde_json::Value, _ctx: &ToolContext) -> Result<ToolResult, ToolFailure> {
        let input: ExitPlanInput = serde_json::from_value(params)?;
        let plan = input.plan.unwrap_or_default();
        Ok(ToolResult::text(format!("Exited plan mode. Plan:\n{}", plan)))
    }
}
