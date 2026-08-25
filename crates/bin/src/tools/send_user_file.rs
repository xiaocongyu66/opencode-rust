//! SendUserFile tool — send a file to the user.
use async_trait::async_trait;
use serde::Deserialize;
use crate::tools::tool::{Tool, ToolContext, ToolFailure, ToolResult};

pub struct SendUserFileTool;
impl SendUserFileTool { pub fn new() -> Self { Self } }

#[derive(Deserialize)]
struct SendUserFileInput {
    #[serde(rename = "file_path")]
    file_path: String,
}

#[async_trait]
impl Tool for SendUserFileTool {
    fn name(&self) -> &str { "SendUserFile" }
    fn description(&self) -> &str { "Sends a file to the user (e.g. as a download)." }
    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "file_path": { "type": "string", "description": "Path to the file to send" }
            },
            "required": ["file_path"]
        })
    }
    async fn execute(&self, params: serde_json::Value, _ctx: &ToolContext) -> Result<ToolResult, ToolFailure> {
        let input: SendUserFileInput = serde_json::from_value(params)?;
        Ok(ToolResult::text(format!("Send file {} to user (not yet implemented)", input.file_path)))
    }
}
