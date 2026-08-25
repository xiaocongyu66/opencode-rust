//! Config tool — view/modify configuration.
use async_trait::async_trait;
use serde::Deserialize;
use crate::tools::tool::{Tool, ToolContext, ToolFailure, ToolResult};

pub struct ConfigTool;
impl ConfigTool { pub fn new() -> Self { Self } }

#[derive(Deserialize)]
struct ConfigInput {
    #[serde(default)]
    key: Option<String>,
    #[serde(default)]
    value: Option<String>,
    #[serde(default)]
    list: Option<bool>,
}

#[async_trait]
impl Tool for ConfigTool {
    fn name(&self) -> &str { "Config" }
    fn description(&self) -> &str {
        "Views or modifies configuration. Pass a key to get its value, or \
         key + value to set. Use list=true to list all config."
    }
    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "key": { "type": "string", "description": "Config key to get/set" },
                "value": { "type": "string", "description": "Value to set (omits = get)" },
                "list": { "type": "boolean", "description": "List all config entries" }
            }
        })
    }
    async fn execute(&self, params: serde_json::Value, _ctx: &ToolContext) -> Result<ToolResult, ToolFailure> {
        let input: ConfigInput = serde_json::from_value(params)?;
        if input.list == Some(true) {
            // Read config.toml and list
            if let Some(home) = dirs::home_dir() {
                let path = home.join(".rsopencode").join("config.toml");
                if let Ok(content) = tokio::fs::read_to_string(&path).await {
                    return Ok(ToolResult::text(content));
                }
            }
            return Ok(ToolResult::text("No config found."));
        }
        match (input.key, input.value) {
            (Some(k), Some(v)) => Ok(ToolResult::text(format!("Set {} = {} (not yet persisted)", k, v))),
            (Some(k), None) => Ok(ToolResult::text(format!("Get {} (config read not yet implemented)", k))),
            _ => Ok(ToolResult::text("Usage: config <key> [value] or config list=true".to_string())),
        }
    }
}
