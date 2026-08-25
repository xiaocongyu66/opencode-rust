//! Edit tool — perform exact string replacements in files.

use async_trait::async_trait;
use serde::Deserialize;
use crate::tool::{Tool, ToolContext, ToolFailure, ToolResult};

pub struct EditTool;

impl EditTool {
    pub fn new() -> Self { Self }
}

#[derive(Deserialize)]
struct EditInput {
    file_path: String,
    old_string: String,
    new_string: String,
    #[serde(default)]
    replace_all: Option<bool>,
}

#[async_trait]
impl Tool for EditTool {
    fn name(&self) -> &str { "edit" }

    fn description(&self) -> &str {
        "Performs exact string replacements in files. The tool will fail if old_string is not found or found multiple times."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "file_path": { "type": "string", "description": "The absolute path to the file to modify" },
                "old_string": { "type": "string", "description": "The exact text to replace" },
                "new_string": { "type": "string", "description": "The replacement text" },
                "replace_all": { "type": "boolean", "description": "Replace all occurrences" }
            },
            "required": ["file_path", "old_string", "new_string"]
        })
    }

    async fn execute(&self, params: serde_json::Value, _ctx: &ToolContext) -> Result<ToolResult, ToolFailure> {
        let input: EditInput = serde_json::from_value(params)?;

        let content = tokio::fs::read_to_string(&input.file_path).await?;
        let count = content.matches(&input.old_string).count();

        if count == 0 {
            return Err(ToolFailure::Message("old_string not found in content".to_string()));
        }
        if count > 1 && input.replace_all != Some(true) {
            return Err(ToolFailure::Message(format!(
                "Found {} matches for old_string. Provide more context or use replace_all.", count
            )));
        }

        let new_content = if input.replace_all == Some(true) {
            content.replace(&input.old_string, &input.new_string)
        } else {
            content.replacen(&input.old_string, &input.new_string, 1)
        };

        tokio::fs::write(&input.file_path, new_content).await?;
        Ok(ToolResult::text(format!("Edit applied to {}", input.file_path)))
    }
}
