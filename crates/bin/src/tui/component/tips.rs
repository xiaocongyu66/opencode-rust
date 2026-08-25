//! Tips component — randomly shows a usage tip with highlighted shortcut text.
//! Ported from tui/src/feature-plugins/home/tips-view.tsx
//!
//! Tips are selected once at construction time (random offset) and remain
//! stable for the lifetime of the component.  Text wrapped in
//! `{highlight}…{/highlight}` is rendered with `theme.text`; the rest uses
//! `theme.text_muted`.  A "● Tip" prefix is shown in `theme.warning`.

use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Wrap};
use ratatui::Frame;

use crate::tui::theme::Theme;

const NO_MODELS_TIP: &str =
    "Run {highlight}/connect{/highlight} to add an AI provider and start coding";

const TIPS: &[&str] = &[
    "Type {highlight}@{/highlight} followed by a filename to fuzzy search and attach files",
    "Start a message with {highlight}!{/highlight} to run shell commands (e.g., {highlight}!ls -la{/highlight})",
    "Press {highlight}Tab{/highlight} to cycle between Build and Plan agents",
    "Use {highlight}/undo{/highlight} to revert the last message and file changes",
    "Use {highlight}/redo{/highlight} to restore previously undone messages and file changes",
    "Run {highlight}/share{/highlight} to create a public opencode.ai link",
    "Drag and drop images or PDFs into the terminal as context",
    "Press {highlight}Ctrl+V{/highlight} to paste images from your clipboard into the prompt",
    "Use {highlight}/editor{/highlight} or {highlight}Ctrl+E{/highlight} to compose messages in your external editor",
    "Run {highlight}/init{/highlight} to auto-generate project rules based on your codebase",
    "Use {highlight}/models{/highlight} or {highlight}Ctrl+M{/highlight} to switch between available AI models",
    "Use {highlight}/themes{/highlight} or {highlight}Ctrl+T{/highlight} to switch between 6 built-in themes",
    "Use {highlight}/new{/highlight} or {highlight}Ctrl+N{/highlight} to start a fresh conversation session",
    "Use {highlight}/sessions{/highlight} or {highlight}Ctrl+S{/highlight} to list, pin, and continue sessions",
    "Press {highlight}p{/highlight} in the session list to pin one at the top",
    "Use {highlight}1{/highlight} through {highlight}9{/highlight} to switch pinned sessions",
    "Run {highlight}/compact{/highlight} to summarize long sessions near context limits",
    "Use {highlight}/export{/highlight} to save the conversation as Markdown",
    "Press {highlight}Ctrl+C{/highlight} to copy the assistant's last message to clipboard",
    "Press {highlight}Ctrl+P{/highlight} to see all available actions and commands",
    "Run {highlight}/connect{/highlight} to add API keys for 75+ supported LLM providers",
    "The leader key is {highlight}Space{/highlight}; combine with other keys for quick actions",
    "Press {highlight}Ctrl+R{/highlight} to quickly switch between recently used models",
    "Press {highlight}Ctrl+B{/highlight} in a session to show or hide the sidebar panel",
    "Use {highlight}PageUp{/highlight}/{highlight}PageDown{/highlight} to navigate through conversation history",
    "Press {highlight}g{/highlight} to jump to the beginning of the conversation",
    "Press {highlight}G{/highlight} to jump to the most recent message",
    "Press {highlight}Alt+Enter{/highlight} to add newlines in your prompt",
    "Press {highlight}Ctrl+U{/highlight} when typing to clear the input field",
    "Press {highlight}Esc{/highlight} to stop the AI mid-response",
    "Switch to {highlight}Plan{/highlight} agent for suggestions without making changes",
    "Use {highlight}@agent-name{/highlight} in prompts to invoke specialized subagents",
    "Use {highlight}({/highlight} / {highlight}){/highlight} / {highlight}[{/highlight} / {highlight}]{/highlight} for parent/child sessions",
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
    "Use patterns like {highlight}\"git *\": \"allow\"{/highlight} for granular bash permissions",
    "Set {highlight}\"rm -rf *\": \"deny\"{/highlight} to block destructive commands",
    "Configure {highlight}\"git push\": \"ask\"{/highlight} to require approval before pushing",
    "Set {highlight}\"formatter\": true{/highlight} to enable built-in formatters",
    "Set {highlight}\"formatter\": false{/highlight} to disable inherited formatters",
    "Define custom formatter commands with file extensions in config",
    "Set {highlight}\"lsp\": true{/highlight} to enable built-in LSP code analysis",
    "Create {highlight}.ts{/highlight} files in {highlight}.opencode/tools/{/highlight} to define new LLM tools",
    "Tool definitions can invoke scripts written in Python, Go, etc",
    "Add {highlight}.ts{/highlight} files to {highlight}.opencode/plugins/{/highlight} for event hooks",
    "Use plugins to send OS notifications when sessions complete",
    "Create a plugin to prevent OpenCode from reading sensitive files",
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
    "Use {highlight}/timeline{/highlight} to jump to specific messages",
    "Press {highlight}c{/highlight} to toggle code block visibility in messages",
    "Use {highlight}/status{/highlight} to see system status info",
    "Enable {highlight}scroll_acceleration{/highlight} in {highlight}tui.json{/highlight} for smooth scrolling",
    "Toggle username display in chat via the command palette",
    "Run {highlight}docker run -it --rm ghcr.io/anomalyco/opencode{/highlight} in a container",
    "Use {highlight}/connect{/highlight} with OpenCode Zen for curated, tested models",
    "Commit your project's {highlight}AGENTS.md{/highlight} file to Git for team sharing",
    "Use {highlight}/review{/highlight} to review uncommitted changes, branches, or PRs",
    "Use {highlight}/help{/highlight} or {highlight}?{/highlight} to show the help dialog",
    "Use {highlight}/rename{/highlight} to rename the current session",
    "Press {highlight}Ctrl+Z{/highlight} to suspend the terminal and return to your shell",
    "Press {highlight}Ctrl+/{/highlight} to undo changes in your prompt",
];

struct TipPart {
    text: String,
    highlight: bool,
}

fn parse_tip(tip: &str) -> Vec<TipPart> {
    let open = "{highlight}";
    let close = "{/highlight}";
    let mut parts = Vec::new();
    let mut remaining = tip;
    let mut in_highlight = false;

    while !remaining.is_empty() {
        if !in_highlight {
            if let Some(pos) = remaining.find(open) {
                if pos > 0 {
                    parts.push(TipPart {
                        text: remaining[..pos].to_string(),
                        highlight: false,
                    });
                }
                remaining = &remaining[pos + open.len()..];
                in_highlight = true;
            } else {
                parts.push(TipPart {
                    text: remaining.to_string(),
                    highlight: false,
                });
                break;
            }
        } else {
            if let Some(pos) = remaining.find(close) {
                parts.push(TipPart {
                    text: remaining[..pos].to_string(),
                    highlight: true,
                });
                remaining = &remaining[pos + close.len()..];
                in_highlight = false;
            } else {
                parts.push(TipPart {
                    text: remaining.to_string(),
                    highlight: true,
                });
                break;
            }
        }
    }

    parts
}

pub struct Tips {
    tip: String,
}

impl Tips {
    pub fn new() -> Self {
        let tip_offset: f64 = {
            let nanos = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.subsec_nanos() as f64 / 1_000_000_000.0)
                .unwrap_or(0.0);
            nanos.fract()
        };
        let idx = (tip_offset * TIPS.len() as f64).floor() as usize;
        let tip = TIPS.get(idx).copied().unwrap_or(NO_MODELS_TIP);
        Self {
            tip: tip.to_string(),
        }
    }

    pub fn with_connected(connected: bool) -> Self {
        if !connected {
            return Self {
                tip: NO_MODELS_TIP.to_string(),
            };
        }
        Self::new()
    }

    pub fn render(&self, f: &mut Frame, area: Rect, theme: &Theme) {
        let parts = parse_tip(&self.tip);

        let mut spans: Vec<Span> = Vec::new();
        spans.push(Span::styled(
            "● Tip ",
            Style::default().fg(theme.warning),
        ));

        for part in &parts {
            let color = if part.highlight {
                theme.text
            } else {
                theme.text_muted
            };
            let style = if part.highlight {
                Style::default().fg(color).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(color)
            };
            spans.push(Span::styled(part.text.clone(), style));
        }

        let line = Line::from(spans);
        let paragraph = Paragraph::new(line)
            .wrap(Wrap { trim: false })
            .alignment(Alignment::Center);

        f.render_widget(paragraph, area);
    }
}

impl Default for Tips {
    fn default() -> Self {
        Self::new()
    }
}
