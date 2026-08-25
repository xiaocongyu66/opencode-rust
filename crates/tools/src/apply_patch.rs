//! Apply patch tool — apply file diffs.

use async_trait::async_trait;
use serde::Deserialize;
use crate::tool::{Tool, ToolContext, ToolFailure, ToolResult};

pub struct ApplyPatchTool;

impl ApplyPatchTool {
    pub fn new() -> Self { Self }
}

#[derive(Deserialize)]
struct ApplyPatchInput {
    patch: String,
}

#[async_trait]
impl Tool for ApplyPatchTool {
    fn name(&self) -> &str { "apply_patch" }

    fn description(&self) -> &str {
        "Use the apply_patch tool to edit files using a stripped-down, file-oriented diff format."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "patch": { "type": "string", "description": "The patch content in the apply_patch format" }
            },
            "required": ["patch"]
        })
    }

    async fn execute(&self, params: serde_json::Value, _ctx: &ToolContext) -> Result<ToolResult, ToolFailure> {
        let _input: ApplyPatchInput = serde_json::from_value(params)?;
        Err(ToolFailure::Message("Patch parsing not yet implemented".to_string()))
    }
}
