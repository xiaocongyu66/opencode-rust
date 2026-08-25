//! Skill tool — invoke a named skill.
//!
//! Aligned with claude-code-best SkillTool:
//! - `skill` (required): skill name (e.g. "commit", "review-pr")
//! - `args` (optional): arguments for the skill

use async_trait::async_trait;
use serde::Deserialize;

use crate::tools::tool::{Tool, ToolContext, ToolFailure, ToolResult};

pub struct SkillTool;

impl SkillTool {
    pub fn new() -> Self {
        Self
    }
}

#[derive(Deserialize)]
struct SkillInput {
    skill: String,
    #[serde(default)]
    args: Option<String>,
}

#[async_trait]
impl Tool for SkillTool {
    fn name(&self) -> &str { "Skill"
    }

    fn description(&self) -> &str {
        "Invokes a named skill (a predefined workflow or command). Skills \
         are registered in the user's config or built-in. Pass the skill \
         name and optional args."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "skill": { "type": "string", "description": "The skill name. E.g., \"commit\", \"review-pr\", or \"pdf\"" },
                "args": { "type": "string", "description": "Optional arguments for the skill" }
            },
            "required": ["skill"]
        })
    }

    async fn execute(
        &self,
        params: serde_json::Value,
        _ctx: &ToolContext,
    ) -> Result<ToolResult, ToolFailure> {
        let input: SkillInput = serde_json::from_value(params)?;
        // Skills registry not yet implemented — return what was requested.
        let args = input.args.unwrap_or_default();
        Ok(ToolResult::text(format!(
            "Skill '{}' requested with args: '{}' (skill execution not yet implemented)",
            input.skill, args
        )))
    }
}
