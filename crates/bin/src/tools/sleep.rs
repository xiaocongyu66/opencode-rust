//! Sleep tool — pause execution for a duration.
//!
//! Aligned with claude-code-best SleepTool:
//! - `duration_seconds` (required): how long to sleep

use async_trait::async_trait;
use serde::Deserialize;

use crate::tools::tool::{Tool, ToolContext, ToolFailure, ToolResult};

pub struct SleepTool;

impl SleepTool {
    pub fn new() -> Self {
        Self
    }
}

#[derive(Deserialize)]
struct SleepInput {
    #[serde(rename = "duration_seconds")]
    duration_seconds: f64,
}

#[async_trait]
impl Tool for SleepTool {
    fn name(&self) -> &str { "Sleep"
    }

    fn description(&self) -> &str {
        "Pauses execution for the given number of seconds. Can be interrupted \
         by the user at any time."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "duration_seconds": {
                    "type": "number",
                    "description": "How long to sleep in seconds."
                }
            },
            "required": ["duration_seconds"]
        })
    }

    async fn execute(
        &self,
        params: serde_json::Value,
        _ctx: &ToolContext,
    ) -> Result<ToolResult, ToolFailure> {
        let input: SleepInput = serde_json::from_value(params)?;
        let secs = input.duration_seconds.max(0.0).min(3600.0);
        tokio::time::sleep(std::time::Duration::from_secs_f64(secs)).await;
        Ok(ToolResult::text(format!(
            "Slept for {:.1} seconds",
            secs
        )))
    }
}
