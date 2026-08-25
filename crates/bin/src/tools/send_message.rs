//! SendMessage tool — send a message to a peer/agent.
use async_trait::async_trait;
use serde::Deserialize;
use crate::tools::tool::{Tool, ToolContext, ToolFailure, ToolResult};

pub struct SendMessageTool;
impl SendMessageTool { pub fn new() -> Self { Self } }

#[derive(Deserialize)]
struct SendMessageInput {
    #[serde(default)]
    target: Option<String>,
    #[serde(default)]
    message: Option<String>,
}

#[async_trait]
impl Tool for SendMessageTool {
    fn name(&self) -> &str { "SendMessage" }
    fn description(&self) -> &str { "Sends a message to a peer agent or team member." }
    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "target": { "type": "string", "description": "Target peer/agent id" },
                "message": { "type": "string", "description": "Message content" }
            }
        })
    }
    async fn execute(&self, params: serde_json::Value, _ctx: &ToolContext) -> Result<ToolResult, ToolFailure> {
        let input: SendMessageInput = serde_json::from_value(params)?;
        Ok(ToolResult::text(format!("Send to {:?}: {:?} (not yet implemented)", input.target, input.message)))
    }
}
