//! RemoteTrigger tool — trigger a remote agent.
use async_trait::async_trait;
use serde::Deserialize;
use crate::tools::tool::{Tool, ToolContext, ToolFailure, ToolResult};

pub struct RemoteTriggerTool;
impl RemoteTriggerTool { pub fn new() -> Self { Self } }

#[derive(Deserialize)]
struct RemoteTriggerInput {
    #[serde(default)]
    url: Option<String>,
    #[serde(default)]
    prompt: Option<String>,
}

#[async_trait]
impl Tool for RemoteTriggerTool {
    fn name(&self) -> &str { "RemoteTrigger" }
    fn description(&self) -> &str {
        "Triggers a remote agent at the given URL with the given prompt."
    }
    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "url": { "type": "string", "description": "Remote agent URL" },
                "prompt": { "type": "string", "description": "Prompt for the remote agent" }
            }
        })
    }
    async fn execute(&self, params: serde_json::Value, _ctx: &ToolContext) -> Result<ToolResult, ToolFailure> {
        let input: RemoteTriggerInput = serde_json::from_value(params)?;
        Ok(ToolResult::text(format!("Remote trigger to {:?} not yet implemented", input.url)))
    }
}
