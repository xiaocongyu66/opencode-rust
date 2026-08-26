//! Tool registry — maps tool names to implementations.
//!
//! Flat registry: tools are stored in a HashMap keyed by name. The
//! `builtin()` constructor registers the 9 core tools aligned with
//! the official Claude Code tool set.

use std::collections::HashMap;

use async_trait::async_trait;

use crate::tools::tool::{Tool, ToolContext, ToolFailure, ToolResult};

/// Registry of available tools.
pub struct ToolRegistry {
    tools: HashMap<String, Box<dyn Tool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    pub fn register(&mut self, tool: Box<dyn Tool>) {
        self.tools.insert(tool.name().to_string(), tool);
    }

    pub fn get(&self, name: &str) -> Option<&dyn Tool> {
        // Direct lookup by primary name (most common path).
        if let Some(t) = self.tools.get(name) {
            return Some(t.as_ref());
        }
        // Fallback: scan aliases (Ch03 "rename is add-only"). Slower but
        // only hits when the primary name wasn't found.
        for t in self.tools.values() {
            if t.aliases().iter().any(|a| *a == name) {
                return Some(t.as_ref());
            }
        }
        None
    }

    /// Layer 1: validate input before permission checks (Ch04 fail fast).
    pub fn validate(&self, name: &str, params: &serde_json::Value) -> Result<(), String> {
        let tool = self
            .get(name)
            .ok_or_else(|| format!("Tool '{}' not found", name))?;
        tool.validate_input(params)
    }

    /// Layer 2-3: tool-specific permission check (Ch04).
    pub fn check_perms(&self, name: &str, params: &serde_json::Value) -> crate::tools::tool::PermissionDecision {
        let tool = match self.get(name) {
            Some(t) => t,
            None => return crate::tools::tool::PermissionDecision::Deny(format!("Tool '{}' not found", name)),
        };
        tool.check_permissions(params)
    }

    /// Tool definitions for the LLM request (name + description + schema).
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

    /// Execute a tool by name.
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

    /// Register the built-in tools aligned with the official Claude Code set.
    pub fn builtin() -> Self {
        let mut registry = Self::new();
        // Core file/shell tools (9)
        registry.register(Box::new(crate::tools::bash::BashTool::new()));
        registry.register(Box::new(crate::tools::read::ReadTool::new()));
        registry.register(Box::new(crate::tools::write::WriteTool::new()));
        registry.register(Box::new(crate::tools::edit::EditTool::new()));
        registry.register(Box::new(crate::tools::glob::GlobTool::new()));
        registry.register(Box::new(crate::tools::grep::GrepTool::new()));
        registry.register(Box::new(crate::tools::webfetch::WebFetchTool::new()));
        registry.register(Box::new(crate::tools::websearch::WebSearchTool::new()));
        registry.register(Box::new(crate::tools::todowrite::TodoWriteTool::new()));
        // Task system (6)
        registry.register(Box::new(crate::tools::task_create::TaskCreateTool::new()));
        registry.register(Box::new(crate::tools::task_get::TaskGetTool::new()));
        registry.register(Box::new(crate::tools::task_list::TaskListTool::new()));
        registry.register(Box::new(crate::tools::task_output::TaskOutputTool::new()));
        registry.register(Box::new(crate::tools::task_stop::TaskStopTool::new()));
        registry.register(Box::new(crate::tools::task_update::TaskUpdateTool::new()));
        // Notebook + shell variants
        registry.register(Box::new(crate::tools::notebook_edit::NotebookEditTool::new()));
        registry.register(Box::new(crate::tools::powershell::PowerShellTool::new()));
        // Planning
        registry.register(Box::new(crate::tools::enter_plan_mode::EnterPlanModeTool::new()));
        registry.register(Box::new(crate::tools::exit_plan_mode::ExitPlanModeTool::new()));
        registry.register(Box::new(crate::tools::verify_plan_execution::VerifyPlanExecutionTool::new()));
        // Agent + skills + questions
        registry.register(Box::new(crate::tools::agent::AgentTool::new()));
        registry.register(Box::new(crate::tools::skill::SkillTool::new()));
        registry.register(Box::new(crate::tools::ask_user_question::AskUserQuestionTool::new()));
        registry.register(Box::new(crate::tools::discover_skills::DiscoverSkillsTool::new()));
        // MCP
        registry.register(Box::new(crate::tools::mcp::McpTool::new()));
        registry.register(Box::new(crate::tools::mcp_auth::McpAuthTool::new()));
        registry.register(Box::new(crate::tools::list_mcp_resources::ListMcpResourcesTool::new()));
        registry.register(Box::new(crate::tools::read_mcp_resource::ReadMcpResourceTool::new()));
        // Cron + worktree + memory
        registry.register(Box::new(crate::tools::schedule_cron::ScheduleCronTool::new()));
        registry.register(Box::new(crate::tools::enter_worktree::EnterWorktreeTool::new()));
        registry.register(Box::new(crate::tools::exit_worktree::ExitWorktreeTool::new()));
        registry.register(Box::new(crate::tools::local_memory_recall::LocalMemoryRecallTool::new()));
        // Monitoring + notifications
        registry.register(Box::new(crate::tools::monitor::MonitorTool::new()));
        registry.register(Box::new(crate::tools::push_notification::PushNotificationTool::new()));
        registry.register(Box::new(crate::tools::remote_trigger::RemoteTriggerTool::new()));
        registry.register(Box::new(crate::tools::review_artifact::ReviewArtifactTool::new()));
        // Communication
        registry.register(Box::new(crate::tools::send_message::SendMessageTool::new()));
        registry.register(Box::new(crate::tools::send_user_file::SendUserFileTool::new()));
        // Code/utility
        registry.register(Box::new(crate::tools::snip::SnipTool::new()));
        registry.register(Box::new(crate::tools::subscribe_pr::SubscribePrTool::new()));
        registry.register(Box::new(crate::tools::suggest_background_pr::SuggestBackgroundPrTool::new()));
        registry.register(Box::new(crate::tools::terminal_capture::TerminalCaptureTool::new()));
        registry.register(Box::new(crate::tools::artifact::ArtifactTool::new()));
        // Teams
        registry.register(Box::new(crate::tools::team::TeamCreateTool::new()));
        registry.register(Box::new(crate::tools::team::TeamDeleteTool::new()));
        // Misc tools
        registry.register(Box::new(crate::tools::ctx_inspect::CtxInspectTool::new()));
        registry.register(Box::new(crate::tools::goal::GoalTool::new()));
        registry.register(Box::new(crate::tools::list_peers::ListPeersTool::new()));
        registry.register(Box::new(crate::tools::brief::BriefTool::new()));
        registry.register(Box::new(crate::tools::execute::ExecuteTool::new()));
        registry.register(Box::new(crate::tools::repl::ReplTool::new()));
        registry.register(Box::new(crate::tools::web_browser::WebBrowserTool::new()));
        registry.register(Box::new(crate::tools::vault_http_fetch::VaultHttpFetchTool::new()));
        registry.register(Box::new(crate::tools::search_extra_tools::SearchExtraToolsTool::new()));
        registry.register(Box::new(crate::tools::execute_extra_tool::ExecuteExtraTool::new()));
        registry.register(Box::new(crate::tools::synthetic_output::SyntheticOutputTool::new()));
        registry.register(Box::new(crate::tools::tungsten::TungstenTool::new()));
        registry.register(Box::new(crate::tools::overflow_test::OverflowTestTool::new()));
        // Config + LSP + sleep
        registry.register(Box::new(crate::tools::config::ConfigTool::new()));
        registry.register(Box::new(crate::tools::lsp::LspTool::new()));
        registry.register(Box::new(crate::tools::sleep::SleepTool::new()));
        registry
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Tool definition sent to the LLM in the API request.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_registers_core_tools() {
        let reg = ToolRegistry::builtin();
        let names: Vec<&str> = reg.tools.keys().map(|s| s.as_str()).collect();
        // Core 9
        for n in &["Bash", "Read", "Write", "Edit", "Glob", "Grep", "WebFetch", "WebSearch", "TodoWrite"] {
            assert!(names.contains(n), "missing tool: {}", n);
        }
        // Task system
        for n in &["TaskCreate", "TaskGet", "TaskList", "TaskOutput", "TaskStop", "TaskUpdate"] {
            assert!(names.contains(n), "missing tool: {}", n);
        }
        // Misc
        for n in &["NotebookEdit", "PowerShell", "Skill", "AskUserQuestion", "Sleep"] {
            assert!(names.contains(n), "missing tool: {}", n);
        }
        assert_eq!(reg.tools.len(), 60, "exactly 60 built-in tools");
    }

    #[test]
    fn definitions_returns_all_tools() {
        let reg = ToolRegistry::builtin();
        let defs = reg.definitions();
        assert_eq!(defs.len(), 60);
        for d in &defs {
            assert!(!d.name.is_empty());
            assert!(!d.description.is_empty());
            assert!(d.input_schema.is_object());
        }
    }
}
