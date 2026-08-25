//! WebBrowser tool — browser automation.
use async_trait::async_trait;
use serde::Deserialize;
use crate::tools::tool::{Tool, ToolContext, ToolFailure, ToolResult};

pub struct WebBrowserTool;
impl WebBrowserTool { pub fn new() -> Self { Self } }

#[derive(Deserialize)]
struct WebBrowserInput {
    #[serde(default)]
    action: Option<String>,
    #[serde(default)]
    url: Option<String>,
    #[serde(default)]
    selector: Option<String>,
}

#[async_trait]
impl Tool for WebBrowserTool {
    fn name(&self) -> &str { "WebBrowser" }
    fn description(&self) -> &str { "Controls a web browser (navigate, click, type, screenshot)." }
    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "action": { "type": "string", "description": "Action: navigate, click, type, screenshot" },
                "url": { "type": "string", "description": "URL to navigate to" },
                "selector": { "type": "string", "description": "CSS selector for click/type" }
            }
        })
    }
    async fn execute(&self, params: serde_json::Value, _ctx: &ToolContext) -> Result<ToolResult, ToolFailure> {
        let input: WebBrowserInput = serde_json::from_value(params)?;
        Ok(ToolResult::text(format!("WebBrowser {:?} (not yet implemented)", input.action)))
    }
}
