//! Tool registry — manages available tools and their definitions.

use std::collections::HashMap;

use crate::tool::{Tool, ToolFailure, ToolResult, ToolContext};

pub struct ToolRegistry {
    tools: HashMap<String, Box<dyn Tool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self { tools: HashMap::new() }
    }

    pub fn register(&mut self, tool: Box<dyn Tool>) {
        let name = tool.name().to_string();
        self.tools.insert(name, tool);
    }

    pub fn get(&self, name: &str) -> Option<&dyn Tool> {
        self.tools.get(name).map(|t| t.as_ref())
    }

    pub fn names(&self) -> Vec<&str> {
        self.tools.keys().map(|s| s.as_str()).collect()
    }

    pub fn definitions(&self) -> Vec<ToolDefinition> {
        self.tools
            .values()
            .map(|t| ToolDefinition {
                name: t.name().to_string(),
                description: t.description().to_string(),
                input_schema: t.parameters_schema(),
            })
            .collect()
    }

    pub async fn execute(
        &self,
        name: &str,
        params: serde_json::Value,
        ctx: &ToolContext,
    ) -> Result<ToolResult, ToolFailure> {
        let tool = self
            .get(name)
            .ok_or_else(|| ToolFailure::Message(format!("Tool '{}' not found", name)))?;
        tool.execute(params, ctx).await
    }

    pub fn builtin() -> Self {
        let mut registry = Self::new();
        registry.register(Box::new(crate::bash::BashTool::new()));
        registry.register(Box::new(crate::edit::EditTool::new()));
        registry.register(Box::new(crate::read::ReadTool::new()));
        registry.register(Box::new(crate::write::WriteTool::new()));
        registry.register(Box::new(crate::glob::GlobTool::new()));
        registry.register(Box::new(crate::grep::GrepTool::new()));
        registry.register(Box::new(crate::webfetch::WebFetchTool::new()));
        registry.register(Box::new(crate::websearch::WebSearchTool::new()));
        registry.register(Box::new(crate::todowrite::TodoWriteTool::new()));
        registry.register(Box::new(crate::question::QuestionTool::new()));
        registry.register(Box::new(crate::skill::SkillTool::new()));
        registry.register(Box::new(crate::apply_patch::ApplyPatchTool::new()));
        registry.register(Box::new(crate::task::TaskTool::new()));
        registry.register(Box::new(crate::code_search::CodeSearchTool::new()));
        registry.register(Box::new(crate::lsp::LspTool::new()));
        registry.register(Box::new(crate::shell::ShellTool::new()));
        registry.register(Box::new(crate::truncate::TruncateTool::new()));
        registry.register(Box::new(crate::external_directory::ExternalDirectoryTool::new()));
        registry
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}
