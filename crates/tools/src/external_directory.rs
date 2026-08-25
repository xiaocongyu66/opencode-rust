//! External directory tool — access directories outside the workspace.

use async_trait::async_trait;
use serde::Deserialize;
use crate::tool::{Tool, ToolContext, ToolFailure, ToolResult};

pub struct ExternalDirectoryTool;

impl ExternalDirectoryTool {
    pub fn new() -> Self { Self }
}

#[derive(Deserialize)]
struct ExternalDirInput {
    path: String,
    #[serde(default)]
    action: Option<String>,
}

#[async_trait]
impl Tool for ExternalDirectoryTool {
    fn name(&self) -> &str { "external_directory" }

    fn description(&self) -> &str {
        "Access and list directories outside the current workspace. Useful for reading system files or referencing external project paths."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": { "type": "string", "description": "Absolute path to the external directory" },
                "action": { "type": "string", "enum": ["list", "read"], "description": "Action to perform. Default: list" }
            },
            "required": ["path"]
        })
    }

    async fn execute(&self, params: serde_json::Value, _ctx: &ToolContext) -> Result<ToolResult, ToolFailure> {
        let input: ExternalDirInput = serde_json::from_value(params)?;
        let action = input.action.as_deref().unwrap_or("list");
        let path = std::path::Path::new(&input.path);

        match action {
            "list" => {
                if !path.is_dir() {
                    return Err(ToolFailure::Message(format!("Not a directory: {}", input.path)));
                }
                let mut entries = Vec::new();
                let mut reader = tokio::fs::read_dir(path).await?;
                while let Some(entry) = reader.next_entry().await? {
                    let name = entry.file_name().to_string_lossy().to_string();
                    let suffix = if entry.file_type().await?.is_dir() { "/" } else { "" };
                    entries.push(format!("{}{}", name, suffix));
                }
                entries.sort();
                Ok(ToolResult::text(entries.join("\n")))
            }
            "read" => {
                if !path.is_file() {
                    return Err(ToolFailure::Message(format!("Not a file: {}", input.path)));
                }
                let content = tokio::fs::read_to_string(path).await?;
                Ok(ToolResult::text(content))
            }
            _ => Err(ToolFailure::Message(format!("Unknown action: {}", action))),
        }
    }
}
