//! Sub-agent definitions (claude-code-book Ch09).
//!
//! Three sources: built-in, plugin, user-defined (Markdown). Four built-in
//! agents cover the common cases: coder, explorer, plan, reviewer. Each
//! carries its own system prompt, allowed/disallowed tools, and limits.

use serde::{Deserialize, Serialize};

/// A sub-agent definition. Mirrors claude-code-book Ch09 BaseAgentDefinition:
/// identity + tool scope + execution control + context management.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseAgentDefinition {
    /// Unique identifier (e.g. "coder", "explorer").
    pub agent_type: String,
    /// When the main agent should delegate to this sub-agent.
    pub when_to_use: String,
    /// Tools this agent is allowed to call. Empty = inherit all.
    #[serde(default)]
    pub allowed_tools: Vec<String>,
    /// Tools this agent must NOT call (denylist).
    #[serde(default)]
    pub disallowed_tools: Vec<String>,
    /// Skills to preload for this agent.
    #[serde(default)]
    pub skills: Vec<String>,
    /// Model override (None = inherit parent's model).
    #[serde(default)]
    pub model: Option<String>,
    /// Max turns before the sub-agent is forced to stop.
    #[serde(default)]
    pub max_turns: Option<u32>,
    /// Whether this agent runs in the background (survives parent turn end).
    #[serde(default)]
    pub background: bool,
    /// Isolation mode: fresh context (don't inherit parent messages).
    #[serde(default)]
    pub isolation_mode: bool,
    /// Omit CLAUDE.md / AGENTS.md from this agent's context.
    #[serde(default)]
    pub omit_project_context: bool,
    /// Max recursion depth for Fork (prevents infinite sub-agent spawning).
    /// Default 3 per claude-code-book Ch09.
    #[serde(default = "default_max_fork_depth")]
    pub max_fork_depth: u32,
}

fn default_max_fork_depth() -> u32 {
    3
}

impl BaseAgentDefinition {
    /// The system prompt for this agent. Built-in agents return a canned
    /// prompt; user-defined agents load from Markdown frontmatter.
    pub fn system_prompt(&self) -> String {
        match self.agent_type.as_str() {
            "coder" => CODER_PROMPT.to_string(),
            "explorer" => EXPLORER_PROMPT.to_string(),
            "plan" => PLAN_PROMPT.to_string(),
            "reviewer" => REVIEWER_PROMPT.to_string(),
            _ => format!(
                "You are a sub-agent of type '{}'. {}",
                self.agent_type, self.when_to_use
            ),
        }
    }
}

/// Four built-in sub-agents (claude-code-book Ch09 §9.1.2).
pub fn builtin_agents() -> Vec<BaseAgentDefinition> {
    vec![
        BaseAgentDefinition {
            agent_type: "coder".into(),
            when_to_use: "Writing or modifying code files. Implements features, fixes bugs, refactors.".into(),
            allowed_tools: vec![
                "Read".into(), "Write".into(), "Edit".into(), "Glob".into(),
                "Grep".into(), "Bash".into(),
            ],
            disallowed_tools: vec!["WebSearch".into(), "WebFetch".into()],
            skills: vec![],
            model: None,
            max_turns: Some(20),
            background: false,
            isolation_mode: false,
            omit_project_context: false,
            max_fork_depth: 3,
        },
        BaseAgentDefinition {
            agent_type: "explorer".into(),
            when_to_use: "Read-only exploration of the codebase. Never modifies files.".into(),
            allowed_tools: vec![
                "Read".into(), "Glob".into(), "Grep".into(), "Bash".into(),
            ],
            disallowed_tools: vec![
                "Write".into(), "Edit".into(), "NotebookEdit".into(),
            ],
            skills: vec![],
            model: None,
            max_turns: Some(15),
            background: false,
            isolation_mode: false,
            omit_project_context: false,
            max_fork_depth: 2,
        },
        BaseAgentDefinition {
            agent_type: "plan".into(),
            when_to_use: "Planning complex tasks before implementation. Produces a plan document.".into(),
            allowed_tools: vec![
                "Read".into(), "Glob".into(), "Grep".into(), "ExitPlanMode".into(),
            ],
            disallowed_tools: vec![
                "Write".into(), "Edit".into(), "Bash".into(),
            ],
            skills: vec![],
            model: None,
            max_turns: Some(10),
            background: false,
            isolation_mode: false,
            omit_project_context: false,
            max_fork_depth: 1,
        },
        BaseAgentDefinition {
            agent_type: "reviewer".into(),
            when_to_use: "Adversarial code review. Finds bugs, security issues, and design flaws.".into(),
            allowed_tools: vec![
                "Read".into(), "Glob".into(), "Grep".into(), "Bash".into(),
            ],
            disallowed_tools: vec![
                "Write".into(), "Edit".into(),
            ],
            skills: vec![],
            model: None,
            max_turns: Some(15),
            background: false,
            isolation_mode: true, // reviewer sees fresh context for objectivity
            omit_project_context: false,
            max_fork_depth: 1,
        },
    ]
}

const CODER_PROMPT: &str = "\
You are a coder sub-agent. Your job is to write and modify code files.
- Focus on implementation; don't get distracted by exploration.
- Always read a file before editing it.
- Prefer Edit over Write when modifying existing files.
- Run tests after changes when possible.
- When done, return a concise summary of what you changed.";

const EXPLORER_PROMPT: &str = "\
You are an explorer sub-agent. Your job is read-only investigation.
- NEVER modify, create, or delete files.
- Use Read, Glob, Grep, and Bash (read-only) to answer questions.
- Return a structured summary of what you found.
- If asked about architecture, trace the call path and cite files:lines.";

const PLAN_PROMPT: &str = "\
You are a plan sub-agent. Your job is to produce an implementation plan.
- Explore the codebase to understand existing patterns.
- Don't write code — only produce a plan document.
- Identify files to change, new files to create, and risks.
- End by calling ExitPlanMode with the plan.";

const REVIEWER_PROMPT: &str = "\
You are a reviewer sub-agent. Your job is adversarial code review.
- Read the changes critically. Assume there are bugs.
- Look for: security issues, race conditions, error handling gaps, \
missing edge cases, API misuse, performance issues.
- Don't fix anything — just report findings with file:line citations.";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_agents_count() {
        let agents = builtin_agents();
        assert_eq!(agents.len(), 4);
        let names: Vec<_> = agents.iter().map(|a| a.agent_type.as_str()).collect();
        assert!(names.contains(&"coder"));
        assert!(names.contains(&"explorer"));
        assert!(names.contains(&"plan"));
        assert!(names.contains(&"reviewer"));
    }

    #[test]
    fn test_explorer_cannot_write() {
        let agents = builtin_agents();
        let explorer = agents.iter().find(|a| a.agent_type == "explorer").unwrap();
        assert!(explorer.disallowed_tools.contains(&"Write".to_string()));
        assert!(explorer.disallowed_tools.contains(&"Edit".to_string()));
    }

    #[test]
    fn test_reviewer_is_isolated() {
        let agents = builtin_agents();
        let reviewer = agents.iter().find(|a| a.agent_type == "reviewer").unwrap();
        assert!(reviewer.isolation_mode, "reviewer should have fresh context");
    }

    #[test]
    fn test_system_prompts_distinct() {
        let agents = builtin_agents();
        let coder = agents.iter().find(|a| a.agent_type == "coder").unwrap();
        let explorer = agents.iter().find(|a| a.agent_type == "explorer").unwrap();
        assert!(coder.system_prompt().contains("coder"));
        assert!(explorer.system_prompt().contains("explorer"));
    }
}
