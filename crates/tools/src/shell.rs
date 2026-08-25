//! Shell tool — persistent shell session for command execution.

use async_trait::async_trait;
use serde::Deserialize;
use crate::tool::{Tool, ToolContext, ToolFailure, ToolResult};

pub struct ShellTool;

impl ShellTool {
    pub fn new() -> Self { Self }
}

#[derive(Deserialize, serde::Serialize)]
struct ShellInput {
    command: String,
    #[serde(default)]
    workdir: Option<String>,
    #[serde(default)]
    timeout: Option<u64>,
}

#[async_trait]
impl Tool for ShellTool {
    fn name(&self) -> &str { "shell" }

    fn description(&self) -> &str {
        "Execute a shell command in a persistent shell session. Maintains working directory and environment between calls."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "command": { "type": "string", "description": "Shell command to execute" },
                "workdir": { "type": "string", "description": "Working directory" },
                "timeout": { "type": "integer", "description": "Timeout in milliseconds" }
            },
            "required": ["command"]
        })
    }

    async fn execute(&self, params: serde_json::Value, ctx: &ToolContext) -> Result<ToolResult, ToolFailure> {
        let input: ShellInput = serde_json::from_value(params)?;
        let result = crate::bash::BashTool::new()
            .execute(serde_json::to_value(&input)?, ctx)
            .await?;
        Ok(result)
    }
}
