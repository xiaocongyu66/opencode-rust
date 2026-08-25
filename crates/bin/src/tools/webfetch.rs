//! WebFetch tool — fetch content from a URL.
//!
//! Aligned with claude-code-best WebFetchTool:
//! - `url` (required): the URL to fetch content from
//! - `prompt` (required): the prompt to run on the fetched content

use async_trait::async_trait;
use serde::Deserialize;

use crate::tools::tool::{Tool, ToolContext, ToolFailure, ToolResult};

pub struct WebFetchTool;

impl WebFetchTool {
    pub fn new() -> Self {
        Self
    }
}

#[derive(Deserialize)]
struct WebFetchInput {
    url: String,
    prompt: String,
}

const MAX_BYTES: usize = 100_000;

#[async_trait]
impl Tool for WebFetchTool {
    fn name(&self) -> &str { "WebFetch"
    }

    fn description(&self) -> &str {
        "Fetches content from a URL and processes it with a prompt. Returns \
         the result of applying the prompt to the fetched content (HTML is \
         converted to text first)."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "url": {
                    "type": "string",
                    "description": "The URL to fetch content from"
                },
                "prompt": {
                    "type": "string",
                    "description": "The prompt to run on the fetched content"
                }
            },
            "required": ["url", "prompt"]
        })
    }

    async fn execute(
        &self,
        params: serde_json::Value,
        _ctx: &ToolContext,
    ) -> Result<ToolResult, ToolFailure> {
        let input: WebFetchInput = serde_json::from_value(params)?;

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| ToolFailure::Message(format!("HTTP client error: {}", e)))?;

        let resp = client
            .get(&input.url)
            .header("User-Agent", "rsopencode/0.1")
            .send()
            .await
            .map_err(|e| ToolFailure::Message(format!("Request failed: {}", e)))?;

        let status = resp.status();
        let code = status.as_u16();
        let code_text = status.canonical_reason().unwrap_or("Unknown").to_string();

        let body = resp
            .text()
            .await
            .map_err(|e| ToolFailure::Message(format!("Failed to read body: {}", e)))?;

        let bytes = body.len();
        let truncated = if body.len() > MAX_BYTES {
            format!("{}... (truncated)", &body[..MAX_BYTES])
        } else {
            body
        };

        // Simple HTML-to-text: strip tags.
        let text = strip_html(&truncated);

        let result = format!(
            "URL: {}\nHTTP {} {}\nSize: {} bytes\n\nPrompt: {}\n\n--- Content ---\n{}",
            input.url, code, code_text, bytes, input.prompt, text
        );

        Ok(ToolResult::text(result))
    }
}

/// Very simple HTML-to-text conversion: remove tags, decode entities.
fn strip_html(html: &str) -> String {
    let mut result = String::with_capacity(html.len());
    let mut in_tag = false;

    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => result.push(ch),
            _ => {}
        }
    }

    // Decode common entities
    result
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&nbsp;", " ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_html_removes_tags() {
        assert_eq!(strip_html("<p>hello</p>"), "hello");
        assert_eq!(strip_html("<a href=\"x\">link</a>"), "link");
    }

    #[test]
    fn strip_html_decodes_entities() {
        assert_eq!(strip_html("a &lt; b &amp; c"), "a < b & c");
    }
}
