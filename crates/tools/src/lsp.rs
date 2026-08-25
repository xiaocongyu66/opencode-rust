//! LSP tool — Language Server Protocol integration.

use async_trait::async_trait;
use crate::tool::{Tool, ToolContext, ToolFailure, ToolResult};

pub struct LspTool;

impl LspTool {
    pub fn new() -> Self { Self }
}

#[async_trait]
impl Tool for LspTool {
    fn name(&self) -> &str { "lsp" }

    fn description(&self) -> &str {
        "Language Server Protocol operations: get definitions, references, diagnostics, and completions."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "action": { "type": "string", "enum": ["definition", "references", "diagnostics", "hover", "completion"] },
                "file_path": { "type": "string" },
                "line": { "type": "integer" },
                "column": { "type": "integer" }
            },
            "required": ["action", "file_path", "line", "column"]
        })
    }

    async fn execute(&self, _params: serde_json::Value, _ctx: &ToolContext) -> Result<ToolResult, ToolFailure> {
        Err(ToolFailure::Message("LSP server not connected".to_string()))
    }
}
