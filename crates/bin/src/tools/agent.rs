//! Agent tool — spawn a subagent for a task.
use async_trait::async_trait;
use serde::Deserialize;
use crate::tools::tool::{Tool, ToolContext, ToolFailure, ToolResult};

pub struct AgentTool;
impl AgentTool { pub fn new() -> Self { Self } }

#[derive(Deserialize)]
struct AgentInput {
    #[serde(default)]
    description: Option<String>,
    #[serde(default, rename = "subagent_type")]
    subagent_type: Option<String>,
    #[serde(default)]
    prompt: Option<String>,
    #[serde(default, rename = "run_in_background")]
    run_in_background: Option<bool>,
}

#[async_trait]
impl Tool for AgentTool {
    fn name(&self) -> &str { "Agent" }
    fn description(&self) -> &str {
        "Spawns a subagent to handle a task. The subagent runs with its own \
         context and tools, and returns a result. Use for delegating \
         complex sub-tasks."
    }
    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "description": { "type": "string", "description": "Short description of what the subagent should do" },
                "subagent_type": { "type": "string", "description": "Type of subagent (e.g. 'general-purpose', 'code-reviewer')" },
                "prompt": { "type": "string", "description": "Full prompt for the subagent" },
                "run_in_background": { "type": "boolean", "description": "Run the subagent in the background" }
            }
        })
    }
    async fn execute(&self, params: serde_json::Value, _ctx: &ToolContext) -> Result<ToolResult, ToolFailure> {
        let input: AgentInput = serde_json::from_value(params)?;
        Ok(ToolResult::text(format!(
            "Subagent spawned (type={:?}, description={:?}) — subagent execution not yet implemented",
            input.subagent_type, input.description
        )))
    }
}
