//! Bash tool — execute shell commands.

use async_trait::async_trait;
use serde::Deserialize;
use crate::tool::{Tool, ToolContext, ToolFailure, ToolResult};

pub struct BashTool;

impl BashTool {
    pub fn new() -> Self { Self }
}

#[derive(Deserialize)]
struct BashInput {
    command: String,
    #[serde(default)]
    workdir: Option<String>,
    #[serde(default)]
    timeout: Option<u64>,
}

const DEFAULT_TIMEOUT_MS: u64 = 120_000;
const MAX_TIMEOUT_MS: u64 = 600_000;
const MAX_CAPTURE_BYTES: usize = 1_048_576;

#[async_trait]
impl Tool for BashTool {
    fn name(&self) -> &str { "bash" }

    fn description(&self) -> &str {
        "Execute a shell command and return its output. Supports optional working directory and timeout."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "command": { "type": "string", "description": "Shell command string to execute" },
                "workdir": { "type": "string", "description": "Working directory" },
                "timeout": { "type": "integer", "description": "Timeout in milliseconds", "maximum": MAX_TIMEOUT_MS }
            },
            "required": ["command"]
        })
    }

    async fn execute(&self, params: serde_json::Value, _ctx: &ToolContext) -> Result<ToolResult, ToolFailure> {
        let input: BashInput = serde_json::from_value(params)?;

        let timeout = input.timeout.unwrap_or(DEFAULT_TIMEOUT_MS).min(MAX_TIMEOUT_MS);
        let shell = if cfg!(windows) { "cmd.exe" } else { "/bin/sh" };

        let mut cmd = tokio::process::Command::new(shell);
        cmd.arg(if cfg!(windows) { "/C" } else { "-c" }).arg(&input.command);

        if let Some(ref dir) = input.workdir {
            cmd.current_dir(dir);
        }

        let output = tokio::time::timeout(
            std::time::Duration::from_millis(timeout),
            cmd.output(),
        ).await;

        match output {
            Ok(Ok(out)) => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                let stderr = String::from_utf8_lossy(&out.stderr);
                let exit_code = out.status.code().unwrap_or(-1);
                let combined = if stderr.is_empty() {
                    stdout.to_string()
                } else {
                    format!("{}\n{}", stdout, stderr)
                };
                let truncated = if combined.len() > MAX_CAPTURE_BYTES {
                    combined[..MAX_CAPTURE_BYTES].to_string()
                } else {
                    combined
                };
                Ok(ToolResult::text(format!("{}\nExit code: {}", truncated, exit_code)))
            }
            Ok(Err(e)) => Err(ToolFailure::Io(e)),
            Err(_) => Ok(ToolResult::text("Command timed out before completion.")),
        }
    }
}
