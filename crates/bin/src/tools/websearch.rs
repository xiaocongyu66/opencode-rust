//! WebSearch tool — search the web.
//!
//! Aligned with claude-code-best WebSearchTool:
//! - `query` (required): the search query
//! - `allowed_domains` (optional): only include results from these domains
//! - `blocked_domains` (optional): never include results from these domains
//! - `num_results` (optional): number of results (default 8)

use async_trait::async_trait;
use serde::Deserialize;

use crate::tools::tool::{Tool, ToolContext, ToolFailure, ToolResult};

pub struct WebSearchTool;

impl WebSearchTool {
    pub fn new() -> Self {
        Self
    }
}

#[derive(Deserialize)]
struct WebSearchInput {
    query: String,
    #[serde(default, rename = "allowed_domains")]
    allowed_domains: Option<Vec<String>>,
    #[serde(default, rename = "blocked_domains")]
    blocked_domains: Option<Vec<String>>,
    #[serde(default, rename = "num_results")]
    num_results: Option<usize>,
}

const DEFAULT_NUM_RESULTS: usize = 8;

#[async_trait]
impl Tool for WebSearchTool {
    fn name(&self) -> &str { "WebSearch"
    }

    fn description(&self) -> &str {
        "Searches the web and returns results. Supports filtering by allowed \
         and blocked domains. Returns titles, URLs, and snippets."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "The search query to use",
                    "minLength": 2
                },
                "allowed_domains": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Only include search results from these domains"
                },
                "blocked_domains": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Never include search results from these domains"
                },
                "num_results": {
                    "type": "number",
                    "description": "Number of search results to return (default: 8)"
                }
            },
            "required": ["query"]
        })
    }

    async fn execute(
        &self,
        params: serde_json::Value,
        _ctx: &ToolContext,
    ) -> Result<ToolResult, ToolFailure> {
        let input: WebSearchInput = serde_json::from_value(params)?;
        let num = input.num_results.unwrap_or(DEFAULT_NUM_RESULTS);

        // Use DuckDuckGo's HTML endpoint as a simple search backend.
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| ToolFailure::Message(format!("HTTP client error: {}", e)))?;

        let url = format!(
            "https://html.duckduckgo.com/html/?q={}",
            urlencoding::encode(&input.query)
        );

        let resp = client
            .get(&url)
            .header("User-Agent", "Mozilla/5.0 (compatible; rsopencode/0.1)")
            .send()
            .await
            .map_err(|e| ToolFailure::Message(format!("Search request failed: {}", e)))?;

        let body = resp
            .text()
            .await
            .map_err(|e| ToolFailure::Message(format!("Failed to read body: {}", e)))?;

        let results = parse_ddg_results(&body);

        // Apply domain filters
        let filtered: Vec<_> = results
            .into_iter()
            .filter(|r| {
                if let Some(ref allowed) = input.allowed_domains {
                    let domain_matches = allowed.iter().any(|d| r.url.contains(d));
                    if !domain_matches {
                        return false;
                    }
                }
                if let Some(ref blocked) = input.blocked_domains {
                    let blocked_match = blocked.iter().any(|d| r.url.contains(d));
                    if blocked_match {
                        return false;
                    }
                }
                true
            })
            .take(num)
            .collect();

        if filtered.is_empty() {
            return Ok(ToolResult::text(format!(
                "No search results for '{}'",
                input.query
            )));
        }

        let mut out = format!("Search results for '{}':\n\n", input.query);
        for (i, r) in filtered.iter().enumerate() {
            out.push_str(&format!("{}. {}\n   {}\n   {}\n\n", i + 1, r.title, r.url, r.snippet));
        }

        Ok(ToolResult::text(out))
    }
}

struct SearchResult {
    title: String,
    url: String,
    snippet: String,
}

/// Parse DuckDuckGo HTML search results.
fn parse_ddg_results(html: &str) -> Vec<SearchResult> {
    let mut results = Vec::new();

    // DDG HTML uses result blocks with class "result" and links with class "result__a"
    // We do a simple regex-based parse.
    let link_re = regex::Regex::new(
        r#"<a[^>]+class="result__a"[^>]*href="([^"]+)"[^>]*>(.*?)</a>"#,
    )
    .unwrap();

    let snippet_re =
        regex::Regex::new(r#"<a[^>]+class="result__snippet"[^>]*>(.*?)</a>"#).unwrap();

    for cap in link_re.captures_iter(html) {
        let url = cap.get(1).map(|m| m.as_str()).unwrap_or("").to_string();
        let title = cap
            .get(2)
            .map(|m| strip_tags(m.as_str()))
            .unwrap_or_default();

        if !url.is_empty() && !title.is_empty() {
            results.push(SearchResult {
                title,
                url: clean_ddg_url(&url),
                snippet: String::new(),
            });
        }
    }

    // Match snippets to results by position
    for (i, cap) in snippet_re.captures_iter(html).enumerate() {
        if i < results.len() {
            results[i].snippet = cap
                .get(1)
                .map(|m| strip_tags(m.as_str()))
                .unwrap_or_default();
        }
    }

    results
}

fn strip_tags(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut in_tag = false;
    for ch in s.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => result.push(ch),
            _ => {}
        }
    }
    result.trim().to_string()
}

/// DDG redirects URLs through a redirect link — extract the actual URL.
fn clean_ddg_url(url: &str) -> String {
    // DDG uses /l/?uddg=<encoded_url> redirects
    if let Some(stripped) = url.strip_prefix("//duckduckgo.com/l/?uddg=") {
        if let Ok(decoded) = urlencoding::decode(stripped) {
            return decoded.to_string();
        }
    }
    url.to_string()
}
