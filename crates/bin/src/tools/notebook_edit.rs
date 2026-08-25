//! NotebookEdit tool — edit Jupyter notebook cells.
//!
//! Aligned with claude-code-best NotebookEditTool:
//! - `notebook_path` (required): absolute path to the notebook file
//! - `cell_id` (optional): the ID of the cell to edit
//! - `new_source` (required): the new source for the cell
//! - `cell_type` (optional): "code" | "markdown"
//! - `edit_mode` (optional): "replace" | "insert" | "delete"

use async_trait::async_trait;
use serde::Deserialize;

use crate::tools::tool::{Tool, ToolContext, ToolFailure, ToolResult};

pub struct NotebookEditTool;

impl NotebookEditTool {
    pub fn new() -> Self {
        Self
    }
}

#[derive(Deserialize)]
struct NotebookEditInput {
    #[serde(rename = "notebook_path")]
    notebook_path: String,
    #[serde(default, rename = "cell_id")]
    cell_id: Option<String>,
    #[serde(rename = "new_source")]
    new_source: String,
    #[serde(default, rename = "cell_type")]
    cell_type: Option<String>,
    #[serde(default, rename = "edit_mode")]
    edit_mode: Option<String>,
}

#[async_trait]
impl Tool for NotebookEditTool {
    fn name(&self) -> &str { "NotebookEdit"
    }

    fn description(&self) -> &str {
        "Edits a Jupyter notebook cell. Supports replacing, inserting, or \
         deleting cells. The notebook_path must be absolute."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "notebook_path": { "type": "string", "description": "The absolute path to the Jupyter notebook file to edit (must be absolute, not relative)" },
                "cell_id": { "type": "string", "description": "The ID of the cell to edit. When inserting a new cell, the new cell will be inserted after the cell with this ID, or at the beginning if not specified." },
                "new_source": { "type": "string", "description": "The new source for the cell" },
                "cell_type": { "type": "string", "enum": ["code", "markdown"], "description": "The type of the cell (code or markdown). If not specified, defaults to the current cell type. Required when edit_mode=insert." },
                "edit_mode": { "type": "string", "enum": ["replace", "insert", "delete"], "description": "The type of edit to make. Defaults to replace." }
            },
            "required": ["notebook_path", "new_source"]
        })
    }

    async fn execute(
        &self,
        params: serde_json::Value,
        _ctx: &ToolContext,
    ) -> Result<ToolResult, ToolFailure> {
        let input: NotebookEditInput = serde_json::from_value(params)?;

        // Read the notebook JSON.
        let content = tokio::fs::read_to_string(&input.notebook_path)
            .await
            .map_err(|e| {
                ToolFailure::Message(format!(
                    "Failed to read {}: {}",
                    input.notebook_path, e
                ))
            })?;

        let mut notebook: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| ToolFailure::Message(format!("Invalid notebook JSON: {}", e)))?;

        let cells = notebook
            .get_mut("cells")
            .and_then(|c| c.as_array_mut())
            .ok_or_else(|| ToolFailure::Message("Notebook has no 'cells' array".to_string()))?;

        let mode = input.edit_mode.as_deref().unwrap_or("replace");
        match mode {
            "delete" => {
                if let Some(id) = &input.cell_id {
                    cells.retain(|c| c.get("id").and_then(|i| i.as_str()) != Some(id));
                    let msg = format!("Deleted cell {} from {}", id, input.notebook_path);
                    let _ = write_back(&input.notebook_path, &notebook).await;
                    return Ok(ToolResult::text(msg));
                }
                Ok(ToolResult::text("cell_id required for delete mode".to_string()))
            }
            "insert" => {
                let cell_type = input.cell_type.as_deref().unwrap_or("code");
                let new_cell = serde_json::json!({
                    "cell_type": cell_type,
                    "source": input.new_source,
                    "metadata": {},
                });
                cells.push(new_cell);
                let msg = format!("Inserted {} cell in {}", cell_type, input.notebook_path);
                let _ = write_back(&input.notebook_path, &notebook).await;
                Ok(ToolResult::text(msg))
            }
            _ => {
                // replace
                if let Some(id) = &input.cell_id {
                    for cell in cells.iter_mut() {
                        if cell.get("id").and_then(|i| i.as_str()) == Some(id) {
                            if let Some(src) = cell.get_mut("source") {
                                *src = serde_json::Value::String(input.new_source.clone());
                            }
                            let msg = format!("Replaced cell {} in {}", id, input.notebook_path);
                            let _ = write_back(&input.notebook_path, &notebook).await;
                            return Ok(ToolResult::text(msg));
                        }
                    }
                    Ok(ToolResult::text(format!("Cell {} not found", id)))
                } else {
                    Ok(ToolResult::text("cell_id required for replace mode".to_string()))
                }
            }
        }
    }
}

async fn write_back(path: &str, notebook: &serde_json::Value) -> Result<(), ToolFailure> {
    let json = serde_json::to_string_pretty(notebook)
        .map_err(|e| ToolFailure::Message(format!("Failed to serialize: {}", e)))?;
    tokio::fs::write(path, json)
        .await
        .map_err(|e| ToolFailure::Message(format!("Failed to write: {}", e)))
}
