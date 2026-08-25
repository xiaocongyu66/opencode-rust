//! ReviewArtifact tool — review an artifact.
use async_trait::async_trait;
use serde::Deserialize;
use crate::tools::tool::{Tool, ToolContext, ToolFailure, ToolResult};

pub struct ReviewArtifactTool;
impl ReviewArtifactTool { pub fn new() -> Self { Self } }

#[derive(Deserialize)]
struct ReviewArtifactInput {
    #[serde(default)]
    artifact_id: Option<String>,
    #[serde(default)]
    url: Option<String>,
}

#[async_trait]
impl Tool for ReviewArtifactTool {
    fn name(&self) -> &str { "ReviewArtifact" }
    fn description(&self) -> &str {
        "Reviews an artifact by id or URL and provides feedback."
    }
    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "artifact_id": { "type": "string", "description": "Artifact id" },
                "url": { "type": "string", "description": "Artifact URL" }
            }
        })
    }
    async fn execute(&self, params: serde_json::Value, _ctx: &ToolContext) -> Result<ToolResult, ToolFailure> {
        let input: ReviewArtifactInput = serde_json::from_value(params)?;
        Ok(ToolResult::text(format!("Review artifact {:?} not yet implemented", input.artifact_id)))
    }
}
