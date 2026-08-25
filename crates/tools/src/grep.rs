//! Grep tool — fast content search using regular expressions.

use async_trait::async_trait;
use serde::Deserialize;
use crate::tool::{Tool, ToolContext, ToolFailure, ToolResult};

pub struct GrepTool;

impl GrepTool {
    pub fn new() -> Self { Self }
}

#[derive(Deserialize)]
struct GrepInput {
    pattern: String,
    #[serde(default)]
    path: Option<String>,
    #[serde(default)]
    include: Option<String>,
}

#[async_trait]
impl Tool for GrepTool {
    fn name(&self) -> &str { "grep" }

    fn description(&self) -> &str {
        "Fast content search tool that works with any codebase size. Searches file contents using regular expressions."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "pattern": { "type": "string", "description": "Regular expression pattern to search for" },
                "path": { "type": "string", "description": "Directory to search in. Defaults to current directory." },
                "include": { "type": "string", "description": "File pattern to include (e.g. \"*.rs\")" }
            },
            "required": ["pattern"]
        })
    }

    async fn execute(&self, params: serde_json::Value, _ctx: &ToolContext) -> Result<ToolResult, ToolFailure> {
        let input: GrepInput = serde_json::from_value(params)?;
        let re = regex::Regex::new(&input.pattern)
            .map_err(|e| ToolFailure::Message(format!("Invalid regex: {}", e)))?;

        let base = input.path.as_deref().unwrap_or(".");
        let include_glob = input.include.as_deref();

        let mut results = Vec::new();
        let base_path = std::path::Path::new(base);
        search_dir(base_path, &re, include_glob, &mut results).await;

        if results.is_empty() {
            Ok(ToolResult::text("No matches found."))
        } else {
            Ok(ToolResult::text(results.join("\n")))
        }
    }
}

async fn search_dir(dir: &std::path::Path, re: &regex::Regex, include: Option<&str>, results: &mut Vec<String>) {
    if results.len() >= 1000 { return; }
    let Ok(mut reader) = tokio::fs::read_dir(dir).await else { return };
    while let Ok(Some(entry)) = reader.next_entry().await {
        let path = entry.path();
        if path.is_dir() {
            if path.file_name().map(|n| n.to_string_lossy().starts_with('.')).unwrap_or(false) { continue; }
            Box::pin(search_dir(&path, re, include, results)).await;
        } else if path.is_file() {
            if let Some(glob_pat) = include {
                if !glob::Pattern::new(glob_pat).map(|p| p.matches(path.file_name().unwrap().to_str().unwrap_or(""))).unwrap_or(false) {
                    continue;
                }
            }
            if let Ok(content) = tokio::fs::read_to_string(&path).await {
                for (i, line) in content.lines().enumerate() {
                    if re.is_match(line) {
                        results.push(format!("{}:{}: {}", path.display(), i + 1, line));
                        if results.len() >= 1000 { return; }
                    }
                }
            }
        }
    }
}
