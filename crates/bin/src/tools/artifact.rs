//! Artifact tool — create/upload an artifact.
use async_trait::async_trait;
use serde::Deserialize;
use crate::tools::tool::{Tool, ToolContext, ToolFailure, ToolResult};

pub struct ArtifactTool;
impl ArtifactTool { pub fn new() -> Self { Self } }

#[derive(Deserialize)]
struct ArtifactInput {
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    content: Option<String>,
    #[serde(default)]
    artifact_type: Option<String>,
}

#[async_trait]
impl Tool for ArtifactTool {
    fn name(&self) -> &str { "Artifact" }
    fn description(&self) -> &str { "Creates an artifact (HTML/report) and uploads it." }
    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "title": { "type": "string", "description": "Artifact title" },
                "content": { "type": "string", "description": "HTML content" },
                "artifact_type": { "type": "string", "description": "Type: html, json, text" }
            }
        })
    }
    async fn execute(&self, params: serde_json::Value, _ctx: &ToolContext) -> Result<ToolResult, ToolFailure> {
        let input: ArtifactInput = serde_json::from_value(params)?;
        Ok(ToolResult::text(format!("Artifact {:?} (not yet implemented)", input.title)))
    }
}
