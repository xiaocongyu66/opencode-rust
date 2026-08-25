//! ScheduleCron tool — schedule a recurring task on a cron schedule.
use async_trait::async_trait;
use serde::Deserialize;
use crate::tools::tool::{Tool, ToolContext, ToolFailure, ToolResult};

pub struct ScheduleCronTool;
impl ScheduleCronTool { pub fn new() -> Self { Self } }

#[derive(Deserialize)]
struct ScheduleCronInput {
    cron: String,
    prompt: String,
    #[serde(default)]
    description: Option<String>,
}

#[async_trait]
impl Tool for ScheduleCronTool {
    fn name(&self) -> &str { "CronCreate" }
    fn description(&self) -> &str {
        "Schedules a recurring task on a cron schedule. The prompt will be \
         executed at the specified times."
    }
    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "cron": { "type": "string", "description": "Cron expression (e.g. '*/5 * * * *')" },
                "prompt": { "type": "string", "description": "Prompt to execute on schedule" },
                "description": { "type": "string", "description": "Human-readable description of the schedule" }
            },
            "required": ["cron", "prompt"]
        })
    }
    async fn execute(&self, params: serde_json::Value, _ctx: &ToolContext) -> Result<ToolResult, ToolFailure> {
        let input: ScheduleCronInput = serde_json::from_value(params)?;
        Ok(ToolResult::text(format!(
            "Scheduled cron '{}' (prompt: '{}') — cron scheduler not yet implemented",
            input.cron, input.prompt
        )))
    }
}
