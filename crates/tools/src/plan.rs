//! Plan tool — planning mode for analysis and code exploration.

use async_trait::async_trait;
use crate::tool::{Tool, ToolContext, ToolFailure, ToolResult};

pub struct PlanTool;

impl PlanTool {
    pub fn new() -> Self { Self }
}

#[async_trait]
impl Tool for PlanTool {
    fn name(&self) -> &str { "plan" }

    fn description(&self) -> &str {
        "Enter or exit planning mode for analysis and code exploration."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {}
        })
    }

    async fn execute(&self, _params: serde_json::Value, _ctx: &ToolContext) -> Result<ToolResult, ToolFailure> {
        Err(ToolFailure::Message("Plan tool not yet implemented".to_string()))
    }
}
