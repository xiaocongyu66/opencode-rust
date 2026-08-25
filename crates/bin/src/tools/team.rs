//! TeamCreate and TeamDelete tools — manage teams.
use async_trait::async_trait;
use serde::Deserialize;
use crate::tools::tool::{Tool, ToolContext, ToolFailure, ToolResult};

pub struct TeamCreateTool;
impl TeamCreateTool { pub fn new() -> Self { Self } }

#[derive(Deserialize)]
struct TeamCreateInput {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    members: Option<Vec<String>>,
}

#[async_trait]
impl Tool for TeamCreateTool {
    fn name(&self) -> &str { "TeamCreate" }
    fn description(&self) -> &str { "Creates a team with the given name and members." }
    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "name": { "type": "string", "description": "Team name" },
                "members": { "type": "array", "items": { "type": "string" }, "description": "Member ids" }
            }
        })
    }
    async fn execute(&self, params: serde_json::Value, _ctx: &ToolContext) -> Result<ToolResult, ToolFailure> {
        let input: TeamCreateInput = serde_json::from_value(params)?;
        Ok(ToolResult::text(format!("Create team {:?} (not yet implemented)", input.name)))
    }
}

pub struct TeamDeleteTool;
impl TeamDeleteTool { pub fn new() -> Self { Self } }

#[derive(Deserialize)]
struct TeamDeleteInput {
    #[serde(default)]
    name: Option<String>,
}

#[async_trait]
impl Tool for TeamDeleteTool {
    fn name(&self) -> &str { "TeamDelete" }
    fn description(&self) -> &str { "Deletes a team by name." }
    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "name": { "type": "string", "description": "Team name to delete" }
            }
        })
    }
    async fn execute(&self, params: serde_json::Value, _ctx: &ToolContext) -> Result<ToolResult, ToolFailure> {
        let input: TeamDeleteInput = serde_json::from_value(params)?;
        Ok(ToolResult::text(format!("Delete team {:?} (not yet implemented)", input.name)))
    }
}
