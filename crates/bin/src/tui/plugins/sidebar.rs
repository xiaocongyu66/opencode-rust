use crate::tui::plugins::builtins::BuiltinTuiPlugin;
use std::collections::HashSet;

pub const SIDEBAR_CONTEXT_ID: &str = "internal:sidebar-context";
pub const SIDEBAR_FILES_ID: &str = "internal:sidebar-files";
pub const SIDEBAR_FOOTER_ID: &str = "internal:sidebar-footer";
pub const SIDEBAR_LSP_ID: &str = "internal:sidebar-lsp";
pub const SIDEBAR_MCP_ID: &str = "internal:sidebar-mcp";
pub const SIDEBAR_TODO_ID: &str = "internal:sidebar-todo";

pub struct SidebarContextPlugin;
pub struct SidebarFilesPlugin;
pub struct SidebarFooterPlugin;
pub struct SidebarLspPlugin;
pub struct SidebarMcpPlugin;
pub struct SidebarTodoPlugin;

impl SidebarContextPlugin {
    pub fn builtin() -> BuiltinTuiPlugin {
        BuiltinTuiPlugin::new(SIDEBAR_CONTEXT_ID).with_order(100)
    }

    pub fn id() -> &'static str { SIDEBAR_CONTEXT_ID }

    pub fn format_cost(cost: u64) -> String {
        format!("${:.2}", cost as f64 / 1_000_000.0)
    }

    pub fn compute_context(tokens: u64, model_limit: Option<u64>) -> Option<u32> {
        model_limit.map(|limit| ((tokens as f64 / limit as f64) * 100.0).round() as u32)
    }

    pub fn render(tokens: u64, percent: Option<u32>, cost: u64) -> Vec<String> {
        let pct = percent.unwrap_or(0);
        vec![
            "Context".to_string(),
            format!("{} tokens", tokens.to_string()),
            format!("{}% used", pct),
            format!("{} spent", Self::format_cost(cost)),
        ]
    }
}

impl SidebarFilesPlugin {
    pub fn builtin() -> BuiltinTuiPlugin {
        BuiltinTuiPlugin::new(SIDEBAR_FILES_ID).with_order(500)
    }

    pub fn id() -> &'static str { SIDEBAR_FILES_ID }

    pub fn change_count_width(additions: u32, deletions: u32) -> usize {
        let parts: Vec<String> = [
            if additions > 0 { format!("+{}", additions) } else { String::new() },
            if deletions > 0 { format!("-{}", deletions) } else { String::new() },
        ]
        .into_iter()
        .filter(|s| !s.is_empty())
        .collect();
        parts.join(" ").len()
    }

    pub fn truncate_left(path: &str, max_width: usize) -> String {
        if path.len() <= max_width { return path.to_string() }
        let prefix = "...";
        let max_width = max_width.max(prefix.len() + 1);
        let start = path.len().saturating_sub(max_width - prefix.len());
        format!("{}{}", prefix, &path[start..])
    }

    pub fn render_file(file: &str, additions: u32, deletions: u32, max_width: usize) -> String {
        let cw = Self::change_count_width(additions, deletions);
        let name = Self::truncate_left(file, (36_usize).saturating_sub(cw).max(2));
        let changes = [
            if additions > 0 { format!("+{}", additions) } else { String::new() },
            if deletions > 0 { format!("-{}", deletions) } else { String::new() },
        ]
        .into_iter()
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
        format!("{} {}", name, changes)
    }
}

impl SidebarFooterPlugin {
    pub fn builtin() -> BuiltinTuiPlugin {
        BuiltinTuiPlugin::new(SIDEBAR_FOOTER_ID).with_order(100)
    }

    pub fn id() -> &'static str { SIDEBAR_FOOTER_ID }

    pub fn should_show_getting_started(has_provider: bool, dismissed: bool) -> bool {
        !has_provider && !dismissed
    }

    pub fn abbreviate_home(path: &str) -> String {
        if let Some(home) = std::env::var("HOME").ok() {
            if path.starts_with(&home) {
                return format!("~{}", &path[home.len()..]);
            }
        }
        path.to_string()
    }

    pub fn split_path(path: &str) -> (String, String) {
        let list: Vec<&str> = path.split('/').collect();
        let name = list.last().copied().unwrap_or("").to_string();
        let parent = if list.len() > 1 {
            list[..list.len() - 1].join("/")
        } else {
            String::new()
        };
        (parent, name)
    }

    pub fn render_path(directory: &str, branch: Option<&str>) -> (String, String) {
        let abbreviated = Self::abbreviate_home(directory);
        let full = match branch {
            Some(b) => format!("{}:{}", abbreviated, b),
            None => abbreviated,
        };
        Self::split_path(&full)
    }
}

impl SidebarLspPlugin {
    pub fn builtin() -> BuiltinTuiPlugin {
        BuiltinTuiPlugin::new(SIDEBAR_LSP_ID).with_order(300)
    }

    pub fn id() -> &'static str { SIDEBAR_LSP_ID }

    pub fn dot_color(status: &str) -> &'static str {
        match status {
            "connected" => "success",
            "failed" => "error",
            "disabled" => "textMuted",
            "needs_auth" => "warning",
            _ => "textMuted",
        }
    }

    pub fn empty_message(lsp_disabled: bool) -> &'static str {
        if lsp_disabled { "LSPs are disabled" } else { "LSPs will activate as files are read" }
    }

    pub fn should_collapse(list_len: usize) -> bool {
        list_len > 2
    }
}

impl SidebarMcpPlugin {
    pub fn builtin() -> BuiltinTuiPlugin {
        BuiltinTuiPlugin::new(SIDEBAR_MCP_ID).with_order(200)
    }

    pub fn id() -> &'static str { SIDEBAR_MCP_ID }

    pub fn dot_color(status: &str) -> &'static str {
        match status {
            "connected" => "success",
            "failed" => "error",
            "disabled" => "textMuted",
            "needs_auth" => "warning",
            "needs_client_registration" => "error",
            _ => "textMuted",
        }
    }

    pub fn status_label(status: &str, error: Option<&str>) -> String {
        match status {
            "connected" => "Connected".to_string(),
            "failed" => error.unwrap_or("Failed").to_string(),
            "disabled" => "Disabled".to_string(),
            "needs_auth" => "Needs auth".to_string(),
            "needs_client_registration" => "Needs client ID".to_string(),
            _ => status.to_string(),
        }
    }

    pub fn count_active(list: &[McpStatus]) -> usize {
        list.iter().filter(|m| m.status == "connected").count()
    }

    pub fn count_errors(list: &[McpStatus]) -> usize {
        list.iter()
            .filter(|m| {
                m.status == "failed"
                    || m.status == "needs_auth"
                    || m.status == "needs_client_registration"
            })
            .count()
    }

    pub fn collapsed_summary(active: usize, errors: usize) -> String {
        let mut s = format!(" ({} active", active);
        if errors > 0 {
            s.push_str(&format!(", {} error{}", errors, if errors > 1 { "s" } else { "" }));
        }
        s.push(')');
        s
    }
}

impl SidebarTodoPlugin {
    pub fn builtin() -> BuiltinTuiPlugin {
        BuiltinTuiPlugin::new(SIDEBAR_TODO_ID).with_order(400)
    }

    pub fn id() -> &'static str { SIDEBAR_TODO_ID }

    pub fn should_show(list: &[TodoItem]) -> bool {
        !list.is_empty() && list.iter().any(|item| item.status != "completed")
    }

    pub fn should_collapse(list_len: usize) -> bool {
        list_len > 2
    }
}

pub struct McpStatus {
    pub name: String,
    pub status: String,
    pub error: Option<String>,
}

pub struct LspStatus {
    pub id: String,
    pub root: String,
    pub status: String,
}

pub struct TodoItem {
    pub status: String,
    pub content: String,
}

pub struct DiffEntry {
    pub file: String,
    pub additions: u32,
    pub deletions: u32,
}

pub type FileTreeExpanded = HashSet<u32>;
