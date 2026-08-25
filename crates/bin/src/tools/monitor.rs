//! Monitor tool — monitor a command's output continuously.
use async_trait::async_trait;
use serde::Deserialize;
use crate::tools::tool::{Tool, ToolContext, ToolFailure, ToolResult};

pub struct MonitorTool;
impl MonitorTool { pub fn new() -> Self { Self } }

#[derive(Deserialize)]
struct MonitorInput {
    command: String,
    #[serde(default)]
    interval_seconds: Option<f64>,
}

#[async_trait]
impl Tool for MonitorTool {
    fn name(&self) -> &str { "Monitor" }
    fn description(&self) -> &str {
        "Runs a command repeatedly at an interval and watches its output. \
         Useful for monitoring test runs, builds, etc."
    }
    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "command": { "type": "string", "description": "The command to monitor" },
                "interval_seconds": { "type": "number", "description": "Interval between runs (default 10)" }
            },
            "required": ["command"]
        })
    }
    async fn execute(&self, params: serde_json::Value, _ctx: &ToolContext) -> Result<ToolResult, ToolFailure> {
        let input: MonitorInput = serde_json::from_value(params)?;
        Ok(ToolResult::text(format!("Monitor '{}' not yet implemented", input.command)))
    }
}
