//! Read tool — read a file or directory from the local filesystem.

use async_trait::async_trait;
use serde::Deserialize;
use crate::tool::{Tool, ToolContext, ToolFailure, ToolResult};

pub struct ReadTool;

impl ReadTool {
    pub fn new() -> Self { Self }
}

#[derive(Deserialize)]
struct ReadInput {
    file_path: String,
    #[serde(default)]
    offset: Option<usize>,
    #[serde(default)]
    limit: Option<usize>,
}

#[async_trait]
impl Tool for ReadTool {
    fn name(&self) -> &str { "read" }

    fn description(&self) -> &str {
        "Read a file or directory from the local filesystem. Returns file contents or directory listing."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "file_path": { "type": "string", "description": "The absolute path to the file to read" },
                "offset": { "type": "integer", "description": "Line number to start reading from (1-indexed)" },
                "limit": { "type": "integer", "description": "Maximum number of lines to read" }
            },
            "required": ["file_path"]
        })
    }

    async fn execute(&self, params: serde_json::Value, _ctx: &ToolContext) -> Result<ToolResult, ToolFailure> {
        let input: ReadInput = serde_json::from_value(params)?;
        let path = std::path::Path::new(&input.file_path);

        if path.is_dir() {
            let mut entries = Vec::new();
            let mut reader = tokio::fs::read_dir(&input.file_path).await?;
            while let Some(entry) = reader.next_entry().await? {
                let name = entry.file_name().to_string_lossy().to_string();
                let suffix = if entry.file_type().await?.is_dir() { "/" } else { "" };
                entries.push(format!("{}{}", name, suffix));
            }
            entries.sort();
            return Ok(ToolResult::text(entries.join("\n")));
        }

        let content = tokio::fs::read_to_string(&input.file_path).await?;
        let lines: Vec<&str> = content.lines().collect();
        let offset = input.offset.unwrap_or(1).saturating_sub(1);
        let limit = input.limit.unwrap_or(2000);

        let selected: Vec<String> = lines
            .iter()
            .skip(offset)
            .take(limit)
            .enumerate()
            .map(|(i, line)| format!("{}: {}", offset + i + 1, line))
            .collect();

        Ok(ToolResult::text(selected.join("\n")))
    }
}
