//! Snip tool — snip/cut a portion of a file or output.
use async_trait::async_trait;
use serde::Deserialize;
use crate::tools::tool::{Tool, ToolContext, ToolFailure, ToolResult};

pub struct SnipTool;
impl SnipTool { pub fn new() -> Self { Self } }

#[derive(Deserialize)]
struct SnipInput {
    #[serde(default, rename = "file_path")]
    file_path: Option<String>,
    #[serde(default)]
    start: Option<usize>,
    #[serde(default)]
    end: Option<usize>,
}

#[async_trait]
impl Tool for SnipTool {
    fn name(&self) -> &str { "Snip" }
    fn description(&self) -> &str { "Snips a portion of a file (lines start..end) for reference." }
    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "file_path": { "type": "string", "description": "File to snip from" },
                "start": { "type": "number", "description": "Start line" },
                "end": { "type": "number", "description": "End line" }
            }
        })
    }
    async fn execute(&self, params: serde_json::Value, _ctx: &ToolContext) -> Result<ToolResult, ToolFailure> {
        let input: SnipInput = serde_json::from_value(params)?;
        Ok(ToolResult::text(format!("Snip {:?} lines {:?}-{:?} (not yet implemented)", input.file_path, input.start, input.end)))
    }
}
