//! Bash tool — execute shell commands.
//!
//! Aligned with claude-code-best BashTool:
//! - `command` (required): the command to execute
//! - `timeout` (optional): timeout in milliseconds (max 600000)
//! - `description` (optional): clear description of what the command does
//! - `run_in_background` (optional): run in background (not yet supported)

use async_trait::async_trait;
use serde::Deserialize;

use crate::tools::tool::{Tool, ToolContext, ToolFailure, ToolResult};

pub struct BashTool;

impl BashTool {
    pub fn new() -> Self {
        Self
    }
}

#[derive(Deserialize)]
struct BashInput {
    command: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    timeout: Option<u64>,
    #[serde(default, rename = "run_in_background")]
    run_in_background: Option<bool>,
}

const DEFAULT_TIMEOUT_MS: u64 = 120_000;
const MAX_TIMEOUT_MS: u64 = 600_000;
const MAX_CAPTURE_BYTES: usize = 1_048_576;

#[async_trait]
impl Tool for BashTool {
    fn name(&self) -> &str { "Bash"
    }

    fn description(&self) -> &str {
        "Executes a given bash command in a shell. The command runs with the \
         user's permissions in their working directory. Use this for running \
         tests, installing packages, or any shell command. Avoid commands \
         that produce very large outputs."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": "The command to execute"
                },
                "timeout": {
                    "type": "number",
                    "description": "Optional timeout in milliseconds (max 600000)",
                    "maximum": MAX_TIMEOUT_MS
                },
                "description": {
                    "type": "string",
                    "description": "Clear, concise description of what this command does in active voice."
                },
                "run_in_background": {
                    "type": "boolean",
                    "description": "Set to true to run this command in the background."
                }
            },
            "required": ["command"]
        })
    }

    async fn execute(
        &self,
        params: serde_json::Value,
        _ctx: &ToolContext,
    ) -> Result<ToolResult, ToolFailure> {
        let input: BashInput = serde_json::from_value(params)?;

        if input.run_in_background == Some(true) {
            return Ok(ToolResult::text(
                "Background commands are not yet supported.",
            ));
        }

        let timeout = input.timeout.unwrap_or(DEFAULT_TIMEOUT_MS).min(MAX_TIMEOUT_MS);
        let shell = if cfg!(windows) {
            std::env::var("COMSPEC").unwrap_or_else(|_| "cmd.exe".to_string())
        } else {
            std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string())
        };

        let mut cmd = tokio::process::Command::new(shell);
        cmd.arg(if cfg!(windows) { "/C" } else { "-c" })
            .arg(&input.command);

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

/// Truncate output to MAX_CAPTURE_BYTES to avoid huge responses.
fn truncate_bytes(s: &str) -> String {
    // Truncate by chars (not bytes) to avoid panicking on UTF-8 boundaries.
    let char_count = s.chars().count();
    if char_count <= MAX_CAPTURE_BYTES {
        return s.to_string();
    }
    let mut truncated: String = s.chars().take(MAX_CAPTURE_BYTES).collect();
    truncated.push_str("\n... (output truncated)");
    truncated
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_keeps_short_strings() {
        assert_eq!(truncate_bytes("hello"), "hello");
    }

    #[test]
    fn truncate_long_strings() {
        let long = "x".repeat(MAX_CAPTURE_BYTES + 100);
        let result = truncate_bytes(&long);
        assert!(result.contains("output truncated"));
        assert!(result.len() < long.len() + 100);
    }
}
