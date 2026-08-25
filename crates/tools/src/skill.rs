//! Skill tool — load a specialized skill.

use async_trait::async_trait;
use serde::Deserialize;
use crate::tool::{Tool, ToolContext, ToolFailure, ToolResult};

pub struct SkillTool;

impl SkillTool {
    pub fn new() -> Self { Self }
}

#[derive(Deserialize)]
struct SkillInput {
    name: String,
}

#[async_trait]
impl Tool for SkillTool {
    fn name(&self) -> &str { "skill" }

    fn description(&self) -> &str {
        "Load a specialized skill when the task matches one of the skills listed in the system prompt."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "name": { "type": "string", "description": "The name of the skill to load" }
            },
            "required": ["name"]
        })
    }

    async fn execute(&self, params: serde_json::Value, _ctx: &ToolContext) -> Result<ToolResult, ToolFailure> {
        let input: SkillInput = serde_json::from_value(params)?;
        Err(ToolFailure::Message(format!("Skill '{}' not found", input.name)))
    }
}
