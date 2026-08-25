//! PushNotification tool — send a push notification.
use async_trait::async_trait;
use serde::Deserialize;
use crate::tools::tool::{Tool, ToolContext, ToolFailure, ToolResult};

pub struct PushNotificationTool;
impl PushNotificationTool { pub fn new() -> Self { Self } }

#[derive(Deserialize)]
struct PushNotificationInput {
    title: String,
    body: String,
}

#[async_trait]
impl Tool for PushNotificationTool {
    fn name(&self) -> &str { "PushNotification" }
    fn description(&self) -> &str {
        "Sends a push notification to the user's devices."
    }
    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "title": { "type": "string", "description": "Notification title" },
                "body": { "type": "string", "description": "Notification body" }
            },
            "required": ["title", "body"]
        })
    }
    async fn execute(&self, params: serde_json::Value, _ctx: &ToolContext) -> Result<ToolResult, ToolFailure> {
        let input: PushNotificationInput = serde_json::from_value(params)?;
        Ok(ToolResult::text(format!("Push: {} — {}", input.title, input.body)))
    }
}
