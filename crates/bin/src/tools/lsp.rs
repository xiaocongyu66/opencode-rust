//! LSP tool — language server protocol operations.
use async_trait::async_trait;
use serde::Deserialize;
use crate::tools::tool::{Tool, ToolContext, ToolFailure, ToolResult};

pub struct LspTool;
impl LspTool { pub fn new() -> Self { Self } }

#[derive(Deserialize)]
struct LspInput {
    #[serde(default)]
    action: Option<String>,
    #[serde(default, rename = "file_path")]
    file_path: Option<String>,
}

#[async_trait]
impl Tool for LspTool {
    fn name(&self) -> &str { "LSP" }
    fn description(&self) -> &str {
        "Interacts with the Language Server Protocol. Supports actions like \
         diagnostics, hover, definition, references for a file."
    }
    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "action": { "type": "string", "description": "LSP action: diagnostics, hover, definition, references" },
                "file_path": { "type": "string", "description": "Path to the file" }
            }
        })
    }
    async fn execute(&self, params: serde_json::Value, _ctx: &ToolContext) -> Result<ToolResult, ToolFailure> {
        let input: LspInput = serde_json::from_value(params)?;
        Ok(ToolResult::text(format!(
            "LSP action '{:?}' on {:?} (LSP integration not yet implemented)",
            input.action, input.file_path
        )))
    }
}
