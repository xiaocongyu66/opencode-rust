//! Tool abstraction — type-safe LLM tool trait.
//!
//! Migrated from `packages/llm/src/tool.ts`.

use async_trait::async_trait;

pub use crate::llm::schema::{ToolDefinition, ToolFailure};

/// Execution context passed to a tool handler.
#[derive(Debug, Clone)]
pub struct ToolExecuteContext {
    pub id: String,
    pub name: String,
}

/// A type-safe LLM tool. Each tool bundles its own description, parameter
/// schema and success schema. The execute handler is optional: omit it when
/// you only want to expose a tool schema to the model and handle tool calls
/// outside this package.
///
/// Errors must be expressed as [`ToolFailure`]. Unmapped errors and defects
/// fail the stream.
#[async_trait]
pub trait Tool: Send + Sync {
    fn description(&self) -> &str;

    /// JSON Schema describing the tool's parameters.
    fn parameters_schema(&self) -> &serde_json::Value;

    /// Optional JSON Schema describing the success value.
    fn success_schema(&self) -> Option<&serde_json::Value> {
        None
    }

    /// Execute the tool with decoded parameters.
    async fn execute(
        &self,
        params: serde_json::Value,
        ctx: ToolExecuteContext,
    ) -> Result<serde_json::Value, ToolFailure>;

    /// Build the canonical [`ToolDefinition`] for this tool.
    fn definition(&self, name: &str) -> ToolDefinition {
        ToolDefinition {
            name: name.to_string(),
            description: self.description().to_string(),
            input_schema: self.parameters_schema().clone(),
            output_schema: self.success_schema().cloned(),
            cache: None,
            metadata: None,
            native: None,
        }
    }
}

/// Convert a map of named tools into the `Vec<ToolDefinition>` shape that
/// `LlmRequest.tools` expects.
pub fn to_definitions(tools: &std::collections::BTreeMap<String, Box<dyn Tool>>) -> Vec<ToolDefinition> {
    tools.iter().map(|(name, tool)| tool.definition(name)).collect()
}
