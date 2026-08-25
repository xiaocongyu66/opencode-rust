//! Write tool — write content to a file.
//!
//! Aligned with claude-code-best FileWriteTool:
//! - `file_path` (required): absolute path to the file
//! - `content` (required): content to write

use async_trait::async_trait;
use serde::Deserialize;

use crate::tools::tool::{Tool, ToolContext, ToolFailure, ToolResult};

pub struct WriteTool;

impl WriteTool {
    pub fn new() -> Self {
        Self
    }
}

#[derive(Deserialize)]
struct WriteInput {
    #[serde(rename = "file_path")]
    file_path: String,
    content: String,
}

#[async_trait]
impl Tool for WriteTool {
    fn name(&self) -> &str { "Write"
    }

    fn description(&self) -> &str {
        "Writes content to a file, creating it if it doesn't exist or \
         overwriting it if it does. The file_path must be absolute."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "file_path": {
                    "type": "string",
                    "description": "The absolute path to the file to write (must be absolute, not relative)"
                },
                "content": {
                    "type": "string",
                    "description": "The content to write to the file"
                }
            },
            "required": ["file_path", "content"]
        })
    }

    async fn execute(
        &self,
        params: serde_json::Value,
        _ctx: &ToolContext,
    ) -> Result<ToolResult, ToolFailure> {
        let input: WriteInput = serde_json::from_value(params)?;

        // Create parent directories if needed.
        if let Some(parent) = std::path::Path::new(&input.file_path).parent() {
            if !parent.as_os_str().is_empty() {
                tokio::fs::create_dir_all(parent).await.ok();
            }
        }

        let existed = std::path::Path::new(&input.file_path).exists();
        tokio::fs::write(&input.file_path, &input.content)
            .await
            .map_err(|e| {
                ToolFailure::Message(format!("Failed to write {}: {}", input.file_path, e))
            })?;

        let action = if existed { "updated" } else { "created" };
        let lines = input.content.lines().count();
        Ok(ToolResult::text(format!(
            "File {} ({} lines, {} bytes)",
            action,
            lines,
            input.content.len()
        )))
    }
}
