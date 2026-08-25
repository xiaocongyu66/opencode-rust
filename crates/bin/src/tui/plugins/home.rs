use crate::tui::plugins::builtins::BuiltinTuiPlugin;
use std::collections::HashMap;

pub const HOME_FOOTER_ID: &str = "internal:home-footer";
pub const HOME_TIPS_ID: &str = "internal:home-tips";

pub struct HomeFooterPlugin;
pub struct HomeTipsPlugin;

impl HomeFooterPlugin {
    pub fn builtin() -> BuiltinTuiPlugin {
        BuiltinTuiPlugin::new(HOME_FOOTER_ID).with_order(100)
    }

    pub fn id() -> &'static str { HOME_FOOTER_ID }

    pub fn render_directory(destination: &str, branch: Option<&str>) -> Option<String> {
        let path = abbreviate_home(destination);
        match branch {
            Some(b) => Some(format!("{}:{}", path, b)),
            None => Some(path),
        }
    }

    pub fn render_mcp(list: &[McpStatus]) -> Option<String> {
        if list.is_empty() { return None }
        let count = list.iter().filter(|m| m.status == "connected").count();
        let has_err = list.iter().any(|m| m.status == "failed");
        let dot = if has_err { "⊙" } else if count > 0 { "⊙" } else { "⊙" };
        Some(format!("{} {} MCP /status", dot, count))
    }

    pub fn render_version(version: &str) -> String { version.to_string() }

    pub fn render_view(dir: Option<String>, mcp: Option<String>, version: &str) -> String {
        let mut parts: Vec<String> = Vec::new();
        if let Some(d) = dir { parts.push(d) }
        if let Some(m) = mcp { parts.push(m) }
        parts.push(String::new());
        parts.push(version.to_string());
        parts.join("  ")
    }
}

impl HomeTipsPlugin {
    pub fn builtin() -> BuiltinTuiPlugin {
        BuiltinTuiPlugin::new(HOME_TIPS_ID).with_order(100)
    }

    pub fn id() -> &'static str { HOME_TIPS_ID }

    pub fn should_show(first: bool, connected: bool, hidden: bool) -> bool {
        (!first || !connected) && !hidden
    }

    pub fn is_connected(providers: &[ProviderInfo]) -> bool {
        providers.iter().any(|p| {
            p.id != "opencode" || p.models.values().any(|m| m.cost.input != 0)
        })
    }

    pub fn render_tip(text: &str, parts: &[TipPart]) -> String {
        parts.iter().map(|p| p.text.as_str()).collect::<Vec<_>>().join("")
    }

    pub fn select_tip(offset: f64, tips: &[String]) -> Option<&String> {
        let index = (offset * tips.len() as f64).floor() as usize;
        tips.get(index.min(tips.len().saturating_sub(1)))
    }
}

pub struct ProviderInfo {
    pub id: String,
    pub models: HashMap<String, ModelCost>,
}

pub struct ModelCost {
    pub input: u64,
}

pub struct McpStatus {
    pub status: String,
}

#[derive(Clone)]
pub struct TipPart {
    pub text: String,
    pub highlight: bool,
}

pub fn parse_tip(tip: &str) -> Vec<TipPart> {
    let mut parts = Vec::new();
    let regex = regex_lite::Regex::new(r"\{highlight\}(.*?)\{/highlight\}").unwrap();
    let mut last_end = 0;
    for m in regex.find_iter(tip) {
        if m.start() > last_end {
            parts.push(TipPart { text: tip[last_end..m.start()].to_string(), highlight: false });
        }
        let inner = &tip[m.start() + 11..m.end() - 12];
        parts.push(TipPart { text: inner.to_string(), highlight: true });
        last_end = m.end();
    }
    if last_end < tip.len() {
        parts.push(TipPart { text: tip[last_end..].to_string(), highlight: false });
    }
    parts
}

pub fn shortcut_text(value: &str) -> String {
    format!("{{highlight}}{}{{/highlight}}", value)
}

pub fn command_text(command: &str, shortcut: &str) -> String {
    if shortcut.is_empty() { return shortcut_text(command) }
    format!("{} or {}", shortcut_text(command), shortcut_text(shortcut))
}

pub fn press(shortcut: &str, text: &str) -> Option<String> {
    if shortcut.is_empty() { return None }
    Some(format!("Press {} {}", shortcut_text(shortcut), text))
}

pub fn abbreviate_home(path: &str) -> String {
    if let Some(home) = std::env::var("HOME").ok() {
        if path.starts_with(&home) {
            return format!("~{}", &path[home.len()..]);
        }
    }
    path.to_string()
}

pub fn builtin_tips() -> Vec<&'static str> {
    vec![
        "Type {highlight}@{/highlight} followed by a filename to fuzzy search and attach files",
        "Start a message with {highlight}!{/highlight} to run shell commands (e.g., {highlight}!ls -la{/highlight})",
        "Use {highlight}/undo{/highlight} to revert the last message and file changes",
        "Use {highlight}/redo{/highlight} to restore previously undone messages and file changes",
        "Run {highlight}/share{/highlight} to create a public opencode.ai link",
        "Drag and drop images or PDFs into the terminal as context",
        "Run {highlight}/init{/highlight} to auto-generate project rules based on your codebase",
        "Run {highlight}/compact{/highlight} to summarize long sessions near context limits",
        "Run {highlight}/connect{/highlight} to add API keys for 75+ supported LLM providers",
        "Switch to {highlight}Plan{/highlight} agent for suggestions without making changes",
        "Use {highlight}@agent-name{/highlight} in prompts to invoke specialized subagents",
        "Create {highlight}opencode.json{/highlight} for server settings, and {highlight}tui.json{/highlight} for TUI",
        "Place TUI settings in {highlight}~/.config/opencode/tui.json{/highlight} for global config",
        "Add {highlight}$schema{/highlight} to your config for autocomplete in your editor",
        "Configure {highlight}model{/highlight} in config to set your default model",
        "Override any keybind in {highlight}tui.json{/highlight} via the {highlight}keybinds{/highlight} section",
        "Set any keybind to {highlight}none{/highlight} to disable it completely",
        "Configure local or remote MCP servers in the {highlight}mcp{/highlight} config section",
        "Add {highlight}.md{/highlight} files to {highlight}.opencode/commands/{/highlight} for reusable prompts",
        "Use {highlight}$ARGUMENTS{/highlight}, {highlight}$1{/highlight}, {highlight}$2{/highlight} in custom commands for dynamic input",
        "Use backticks to inject shell output (e.g., {highlight}`git status`{/highlight})",
        "Add {highlight}.md{/highlight} files to {highlight}.opencode/agents/{/highlight} for specialized AI personas",
        "Configure per-agent permissions for {highlight}edit{/highlight}, {highlight}bash{/highlight}, and {highlight}webfetch{/highlight} tools",
        "Use {highlight}opencode run{/highlight} for non-interactive scripting",
        "Use {highlight}opencode --continue{/highlight} to resume the last session",
        "Use {highlight}opencode run -f file.ts{/highlight} to attach files via CLI",
        "Use {highlight}--format json{/highlight} for machine-readable output in scripts",
        "Run {highlight}opencode serve{/highlight} for headless API access to OpenCode",
        "Use {highlight}opencode run --attach{/highlight} to connect to a running server",
        "Run {highlight}opencode upgrade{/highlight} to update to the latest version",
        "Run {highlight}opencode auth list{/highlight} to see all configured providers",
        "Run {highlight}opencode agent create{/highlight} for guided agent creation",
        "Use {highlight}/opencode{/highlight} in GitHub issues/PRs to trigger AI actions",
        "Run {highlight}opencode github install{/highlight} to set up the GitHub workflow",
        "Comment {highlight}/opencode fix this{/highlight} on issues to auto-create PRs",
        "Comment {highlight}/oc{/highlight} on PR code lines for targeted code reviews",
        "Use {highlight}\"theme\": \"system\"{/highlight} to match your terminal's colors",
        "Create JSON theme files in {highlight}.opencode/themes/{/highlight} directory",
        "Themes support dark/light variants for both modes",
        "Use numeric xterm color codes 0-255 in custom theme JSON",
        "Use {highlight}{env:VAR_NAME}{/highlight} for environment variables in config",
        "Use {highlight}{file:path}{/highlight} to include file contents in config values",
        "Use {highlight}instructions{/highlight} in config to load additional rules files",
        "Set agent {highlight}temperature{/highlight} from 0.0 (focused) to 1.0 (creative)",
        "Configure {highlight}steps{/highlight} to limit agentic iterations per request",
        "Set {highlight}\"tools\": {\"bash\": false}{/highlight} to disable specific tools",
        "Set {highlight}\"mcp_*\": false{/highlight} to disable all tools from an MCP server",
        "Override global tool settings per agent configuration",
        "Set {highlight}\"share\": \"auto\"{/highlight} to automatically share all sessions",
        "Set {highlight}\"share\": \"disabled\"{/highlight} to prevent any session sharing",
        "Run {highlight}/unshare{/highlight} to remove a session from public access",
        "Permission {highlight}doom_loop{/highlight} prevents infinite tool call loops",
        "Permission {highlight}external_directory{/highlight} protects files outside project",
        "Run {highlight}opencode debug config{/highlight} to troubleshoot configuration",
        "Use {highlight}--print-logs{/highlight} flag to see detailed logs in stderr",
        "Enable {highlight}scroll_acceleration{/highlight} in {highlight}tui.json{/highlight} for smooth scrolling",
        "Run {highlight}docker run -it --rm ghcr.io/anomalyco/opencode{/highlight} in a container",
        "Use {highlight}/connect{/highlight} with OpenCode Zen for curated, tested models",
        "Commit your project's {highlight}AGENTS.md{/highlight} file to Git for team sharing",
        "Use {highlight}/review{/highlight} to review uncommitted changes, branches, or PRs",
        "Use {highlight}/rename{/highlight} to rename the current session",
    ]
}

pub const NO_MODELS_TIP: &str =
    "Run {highlight}/connect{/highlight} to add an AI provider and start coding";

pub const INPUT_UNDO_TIP: &str = "Press {highlight}{/highlight} to undo changes in your prompt";

pub const TERMINAL_SUSPEND_TIP: &str =
    "Press {highlight}{/highlight} to suspend the terminal and return to your shell";
