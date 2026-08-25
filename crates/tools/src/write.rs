//! Write tool — write a file to the local filesystem.

use async_trait::async_trait;
use serde::Deserialize;
use crate::tool::{Tool, ToolContext, ToolFailure, ToolResult};

pub struct WriteTool;

impl WriteTool {
    pub fn new() -> Self { Self }
}

#[derive(Deserialize)]
struct WriteInput {
    file_path: String,
    content: String,
}

#[async_trait]
impl Tool for WriteTool {
    fn name(&self) -> &str { "write" }

    fn description(&self) -> &str {
        "Writes a file to the local filesystem. Overwrites the existing file if one exists."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "file_path": { "type": "string", "description": "The absolute path to the file to write" },
                "content": { "type": "string", "description": "The content to write to the file" }
            },
            "required": ["file_path", "content"]
        })
    }

    async fn execute(&self, params: serde_json::Value, _ctx: &ToolContext) -> Result<ToolResult, ToolFailure> {
        let input: WriteInput = serde_json::from_value(params)?;

        if let Some(parent) = std::path::Path::new(&input.file_path).parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        tokio::fs::write(&input.file_path, &input.content).await?;
        Ok(ToolResult::text(format!("File written: {}", input.file_path)))
    }
}
