//! WebFetch tool — fetch content from a URL.

use async_trait::async_trait;
use serde::Deserialize;
use crate::tool::{Tool, ToolContext, ToolFailure, ToolResult};

pub struct WebFetchTool;

impl WebFetchTool {
    pub fn new() -> Self { Self }
}

#[derive(Deserialize)]
struct WebFetchInput {
    url: String,
    #[serde(default)]
    format: Option<String>,
}

#[async_trait]
impl Tool for WebFetchTool {
    fn name(&self) -> &str { "webfetch" }

    fn description(&self) -> &str {
        "Fetches content from a specified URL and converts to the requested format (markdown by default)."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "url": { "type": "string", "description": "The URL to fetch" },
                "format": { "type": "string", "description": "Output format: markdown, text, or html" }
            },
            "required": ["url"]
        })
    }

    async fn execute(&self, params: serde_json::Value, _ctx: &ToolContext) -> Result<ToolResult, ToolFailure> {
        let input: WebFetchInput = serde_json::from_value(params)?;

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| ToolFailure::Message(e.to_string()))?;

        let resp = client.get(&input.url).send().await
            .map_err(|e| ToolFailure::Message(format!("Request failed: {}", e)))?;

        let content = resp.text().await
            .map_err(|e| ToolFailure::Message(format!("Failed to read response: {}", e)))?;

        let format = input.format.as_deref().unwrap_or("markdown");
        let result = match format {
            "html" => content,
            "text" => strip_html_tags(&content),
            _ => strip_html_tags(&content),
        };

        Ok(ToolResult::text(result))
    }
}

fn strip_html_tags(html: &str) -> String {
    let mut result = String::with_capacity(html.len());
    let mut in_tag = false;
    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            c if !in_tag => result.push(c),
            _ => {}
        }
    }
    result
}
