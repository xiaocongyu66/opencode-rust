//! Glob tool — fast file pattern matching.

use async_trait::async_trait;
use serde::Deserialize;
use crate::tool::{Tool, ToolContext, ToolFailure, ToolResult};

pub struct GlobTool;

impl GlobTool {
    pub fn new() -> Self { Self }
}

#[derive(Deserialize)]
struct GlobInput {
    pattern: String,
    #[serde(default)]
    path: Option<String>,
}

#[async_trait]
impl Tool for GlobTool {
    fn name(&self) -> &str { "glob" }

    fn description(&self) -> &str {
        "Fast file pattern matching tool that works with any codebase size. Supports glob patterns like \"**/*.js\" or \"src/**/*.ts\"."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "pattern": { "type": "string", "description": "Glob pattern to match files" },
                "path": { "type": "string", "description": "Directory to search in. Defaults to current directory." }
            },
            "required": ["pattern"]
        })
    }

    async fn execute(&self, params: serde_json::Value, _ctx: &ToolContext) -> Result<ToolResult, ToolFailure> {
        let input: GlobInput = serde_json::from_value(params)?;
        let base = input.path.as_deref().unwrap_or(".");
        let pattern = if base == "." {
            input.pattern.clone()
        } else {
            format!("{}/{}", base.trim_end_matches('/'), input.pattern)
        };

        let mut results: Vec<String> = glob::glob(&pattern)
            .map_err(|e| ToolFailure::Message(format!("Invalid pattern: {}", e)))?
            .filter_map(|entry| entry.ok())
            .map(|p| p.to_string_lossy().to_string())
            .collect();
        results.sort();

        if results.is_empty() {
            Ok(ToolResult::text("No files matched the pattern."))
        } else {
            Ok(ToolResult::text(results.join("\n")))
        }
    }
}
