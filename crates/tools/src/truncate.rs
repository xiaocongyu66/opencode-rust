//! Truncate tool — truncate large outputs to a manageable size.

use async_trait::async_trait;
use serde::Deserialize;
use crate::tool::{Tool, ToolContext, ToolFailure, ToolResult};

pub struct TruncateTool;

impl TruncateTool {
    pub fn new() -> Self { Self }
}

#[derive(Deserialize)]
struct TruncateInput {
    text: String,
    #[serde(default = "default_max_chars")]
    max_chars: usize,
    #[serde(default)]
    lines: Option<usize>,
}

fn default_max_chars() -> usize { 50_000 }

#[async_trait]
impl Tool for TruncateTool {
    fn name(&self) -> &str { "truncate" }

    fn description(&self) -> &str {
        "Truncate text output to a manageable size. Useful for preventing context overflow when dealing with large command outputs."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "text": { "type": "string", "description": "Text to truncate" },
                "max_chars": { "type": "integer", "description": "Maximum characters to keep. Default: 50000" },
                "lines": { "type": "integer", "description": "Maximum lines to keep (overrides max_chars if set)" }
            },
            "required": ["text"]
        })
    }

    async fn execute(&self, params: serde_json::Value, _ctx: &ToolContext) -> Result<ToolResult, ToolFailure> {
        let input: TruncateInput = serde_json::from_value(params)?;

        let result = if let Some(max_lines) = input.lines {
            let lines: Vec<&str> = input.text.lines().collect();
            if lines.len() <= max_lines {
                input.text
            } else {
                let half = max_lines / 2;
                let head: Vec<&str> = lines.iter().take(half).copied().collect();
                let tail: Vec<&str> = lines.iter().skip(lines.len() - half).copied().collect();
                format!("{}\n... ({} lines truncated) ...\n{}", head.join("\n"), lines.len() - max_lines, tail.join("\n"))
            }
        } else if input.text.len() <= input.max_chars {
            input.text
        } else {
            let half = input.max_chars / 2;
            format!("{}... ({} chars truncated) ...{}", &input.text[..half], input.text.len() - input.max_chars, &input.text[input.text.len() - half..])
        };

        Ok(ToolResult::text(result))
    }
}
