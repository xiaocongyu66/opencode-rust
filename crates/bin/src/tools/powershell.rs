//! PowerShell tool — execute PowerShell commands (Windows).
//!
//! Aligned with claude-code-best PowerShellTool (same schema as Bash but
//! uses powershell.exe on Windows). On non-Windows, delegates to Bash.

use async_trait::async_trait;
use serde::Deserialize;

use crate::tools::tool::{Tool, ToolContext, ToolFailure, ToolResult};

pub struct PowerShellTool;

impl PowerShellTool {
    pub fn new() -> Self {
        Self
    }
}

#[derive(Deserialize)]
struct PowerShellInput {
    command: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    timeout: Option<u64>,
}

const DEFAULT_TIMEOUT_MS: u64 = 120_000;
const MAX_TIMEOUT_MS: u64 = 600_000;
const MAX_CAPTURE_BYTES: usize = 1_048_576;

#[async_trait]
impl Tool for PowerShellTool {
    fn name(&self) -> &str { "PowerShell"
    }

    fn description(&self) -> &str {
        "Executes a PowerShell command (Windows). On non-Windows platforms, \
         delegates to the bash tool."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "command": { "type": "string", "description": "The PowerShell command to execute" },
                "timeout": { "type": "number", "description": "Optional timeout in milliseconds (max 600000)", "maximum": MAX_TIMEOUT_MS },
                "description": { "type": "string", "description": "Clear, concise description of what this command does." }
            },
            "required": ["command"]
        })
    }

    async fn execute(
        &self,
        params: serde_json::Value,
        _ctx: &ToolContext,
    ) -> Result<ToolResult, ToolFailure> {
        let input: PowerShellInput = serde_json::from_value(params)?;
        let timeout = input.timeout.unwrap_or(DEFAULT_TIMEOUT_MS).min(MAX_TIMEOUT_MS);

        let shell = if cfg!(windows) {
            "powershell.exe".to_string()
        } else {
            // On non-Windows, fall back to the user's shell.
            std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string())
        };

        let mut cmd = tokio::process::Command::new(shell);
        if cfg!(windows) {
            cmd.arg("-NoProfile").arg("-Command").arg(&input.command);
        } else {
            cmd.arg("-c").arg(&input.command);
        }

        let output = tokio::time::timeout(
            std::time::Duration::from_millis(timeout),
            cmd.output(),
        )
        .await;

        match output {
            Ok(Ok(out)) => {
                let stdout = truncate_bytes(&String::from_utf8_lossy(&out.stdout));
                let stderr = truncate_bytes(&String::from_utf8_lossy(&out.stderr));
                let exit = out.status.code().unwrap_or(-1);

                let mut result = String::new();
                if !stdout.is_empty() {
                    result.push_str(&stdout);
                }
                if !stderr.is_empty() {
                    if !result.is_empty() {
                        result.push_str("\n--- stderr ---\n");
                    }
                    result.push_str(&stderr);
                }
                result.push_str(&format!("\n[exit: {}]", exit));
                Ok(ToolResult::text(result.trim().to_string()))
            }
            Ok(Err(e)) => Ok(ToolResult::text(format!(
                "Failed to execute command: {}",
                e
            ))),
            Err(_) => Ok(ToolResult::text(format!(
                "Command timed out after {}ms",
                timeout
            ))),
        }
    }
}

fn truncate_bytes(s: &str) -> String {
    let char_count = s.chars().count();
    if char_count <= MAX_CAPTURE_BYTES {
        return s.to_string();
    }
    let mut truncated: String = s.chars().take(MAX_CAPTURE_BYTES).collect();
    truncated.push_str("\n... (output truncated)");
    truncated
}
