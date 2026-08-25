//! DiscoverSkills tool — discover available skills.
use async_trait::async_trait;
use crate::tools::tool::{Tool, ToolContext, ToolFailure, ToolResult};

pub struct DiscoverSkillsTool;
impl DiscoverSkillsTool { pub fn new() -> Self { Self } }

#[async_trait]
impl Tool for DiscoverSkillsTool {
    fn name(&self) -> &str { "DiscoverSkills" }
    fn description(&self) -> &str { "Discovers available skills from the skill registry." }
    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({"type": "object", "properties": {}})
    }
    async fn execute(&self, _params: serde_json::Value, _ctx: &ToolContext) -> Result<ToolResult, ToolFailure> {
        Ok(ToolResult::text("No skills discovered (skill registry not yet implemented)"))
    }
}
