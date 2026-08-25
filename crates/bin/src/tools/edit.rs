//! Edit tool — find and replace text in a file.
//!
//! Aligned with claude-code-best FileEditTool:
//! - `file_path` (required): absolute path to the file to modify
//! - `old_string` (required): the text to replace
//! - `new_string` (required): the text to replace it with
//! - `replace_all` (optional): replace all occurrences (default false)

use async_trait::async_trait;
use serde::Deserialize;

use crate::tools::tool::{Tool, ToolContext, ToolFailure, ToolResult};

pub struct EditTool;

impl EditTool {
    pub fn new() -> Self {
        Self
    }
}

#[derive(Deserialize)]
struct EditInput {
    #[serde(rename = "file_path")]
    file_path: String,
    #[serde(rename = "old_string")]
    old_string: String,
    #[serde(rename = "new_string")]
    new_string: String,
    #[serde(default, rename = "replace_all")]
    replace_all: Option<bool>,
}

#[async_trait]
impl Tool for EditTool {
    fn name(&self) -> &str { "Edit"
    }

    fn description(&self) -> &str {
        "Performs an exact string replacement in a file. The old_string must \
         be unique in the file unless replace_all is true. The new_string \
         must be different from old_string."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "file_path": {
                    "type": "string",
                    "description": "The absolute path to the file to modify"
                },
                "old_string": {
                    "type": "string",
                    "description": "The text to replace"
                },
                "new_string": {
                    "type": "string",
                    "description": "The text to replace it with (must be different from old_string)"
                },
                "replace_all": {
                    "type": "boolean",
                    "description": "Replace all occurrences of old_string (default false)"
                }
            },
            "required": ["file_path", "old_string", "new_string"]
        })
    }

    async fn execute(
        &self,
        params: serde_json::Value,
        _ctx: &ToolContext,
    ) -> Result<ToolResult, ToolFailure> {
        let input: EditInput = serde_json::from_value(params)?;

        if input.old_string == input.new_string {
            return Ok(ToolResult::text(
                "old_string and new_string must be different",
            ));
        }

        let content = tokio::fs::read_to_string(&input.file_path)
            .await
            .map_err(|e| {
                ToolFailure::Message(format!("Failed to read {}: {}", input.file_path, e))
            })?;

        let replace_all = input.replace_all.unwrap_or(false);
        let count = if replace_all {
            content.matches(&input.old_string).count()
        } else {
            // For single replace, old_string must be unique.
            let matches: Vec<usize> = content
                .match_indices(&input.old_string)
                .map(|(i, _)| i)
                .collect();
            if matches.is_empty() {
                return Ok(ToolResult::text(format!(
                    "old_string not found in {}",
                    input.file_path
                )));
            }
            if matches.len() > 1 {
                return Ok(ToolResult::text(format!(
                    "old_string appears {} times in {} — use replace_all=true or provide more context to make it unique",
                    matches.len(),
                    input.file_path
                )));
            }
            1
        };

        let new_content = if replace_all {
            content.replace(&input.old_string, &input.new_string)
        } else {
            content.replacen(&input.old_string, &input.new_string, 1)
        };

        tokio::fs::write(&input.file_path, &new_content)
            .await
            .map_err(|e| {
                ToolFailure::Message(format!("Failed to write {}: {}", input.file_path, e))
            })?;

        Ok(ToolResult::text(format!(
            "Replaced {} occurrence(s) in {}",
            count,
            input.file_path
        )))
    }
}
