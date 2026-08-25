//! Read tool — read file contents.
//!
//! Aligned with claude-code-best FileReadTool:
//! - `file_path` (required): absolute path to the file
//! - `offset` (optional): line number to start reading from
//! - `limit` (optional): number of lines to read

use async_trait::async_trait;
use serde::Deserialize;

use crate::tools::tool::{Tool, ToolContext, ToolFailure, ToolResult};

pub struct ReadTool;

impl ReadTool {
    pub fn new() -> Self {
        Self
    }
}

#[derive(Deserialize)]
struct ReadInput {
    #[serde(rename = "file_path")]
    file_path: String,
    #[serde(default)]
    offset: Option<usize>,
    #[serde(default)]
    limit: Option<usize>,
}

const DEFAULT_LIMIT: usize = 2000;
const MAX_LINE_LENGTH: usize = 2000;

#[async_trait]
impl Tool for ReadTool {
    fn name(&self) -> &str { "Read"
    }

    fn description(&self) -> &str {
        "Reads the contents of a file. Supports reading specific line ranges \
         with offset and limit. Lines are prefixed with line numbers."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "file_path": {
                    "type": "string",
                    "description": "The absolute path to the file to read"
                },
                "offset": {
                    "type": "integer",
                    "description": "The line number to start reading from. Only provide if the file is too large to read at once.",
                    "minimum": 0
                },
                "limit": {
                    "type": "integer",
                    "description": "The number of lines to read. Only provide if the file is too large to read at once.",
                    "minimum": 1
                }
            },
            "required": ["file_path"]
        })
    }

    async fn execute(
        &self,
        params: serde_json::Value,
        _ctx: &ToolContext,
    ) -> Result<ToolResult, ToolFailure> {
        let input: ReadInput = serde_json::from_value(params)?;

        let content = tokio::fs::read_to_string(&input.file_path)
            .await
            .map_err(|e| {
                ToolFailure::Message(format!("Failed to read {}: {}", input.file_path, e))
            })?;

        let lines: Vec<&str> = content.lines().collect();
        let total = lines.len();

        let offset = input.offset.unwrap_or(0).min(total);
        let limit = input.limit.unwrap_or(DEFAULT_LIMIT);
        let end = (offset + limit).min(total);

        let mut result = String::new();
        result.push_str(&format!("{}-{} of {} lines\n", offset + 1, end, total));
        for (i, line) in lines[offset..end].iter().enumerate() {
            let line_no = offset + i + 1;
            let truncated = if line.len() > MAX_LINE_LENGTH {
                format!("{}... (truncated)", &line[..MAX_LINE_LENGTH])
            } else {
                line.to_string()
            };
            result.push_str(&format!("{:>6}\t{}\n", line_no, truncated));
        }

        if end < total {
            result.push_str(&format!("{} more lines below\n", total - end));
        }

        Ok(ToolResult::text(result))
    }
}
