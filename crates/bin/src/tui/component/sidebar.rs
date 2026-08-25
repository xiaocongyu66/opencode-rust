//! Sidebar component — 还原原版 routes/session/sidebar.tsx + feature-plugins/sidebar/
//! 右侧固定 42 宽，显示 Context/MCP/LSP/Files/Todo

use ratatui::layout::Rect;
use ratatui::style::{Style, Modifier};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;
use crate::tui::theme::Theme;

pub struct Sidebar {
    pub title: String,
    pub workspace: String,
    pub directory: String,
    pub version: String,
    pub todos: Vec<(String, String)>,
    pub context_tokens: u64,
    pub context_percent: u32,
    pub context_cost: f64,
    /// Maximum context window for the current model (0 = unknown).
    pub context_limit: u64,
    /// Number of LLM turns (steps) in the session.
    pub step_count: u64,
    /// Number of tool calls made.
    pub tool_call_count: u64,
    pub lsp_connected: usize,
    pub lsp_disabled: bool,
    pub mcp_connected: usize,
    pub mcp_errors: usize,
}

impl Sidebar {
    pub fn new() -> Self {
        Self {
            title: crate::core::session::default_parent_title(),
            workspace: "default".to_string(),
            directory: std::env::current_dir()
                .map(|p| {
                    let s = p.to_string_lossy().to_string();
                    if let Some(home) = std::env::var("HOME").ok() {
                        if s.starts_with(&home) { return format!("~{}", &s[home.len()..]); }
                    }
                    s
                })
                .unwrap_or_default(),
            version: "0.1.0".to_string(),
            todos: vec![],
            context_tokens: 0,
            context_percent: 0,
            context_cost: 0.0,
            context_limit: 0,
            step_count: 0,
            tool_call_count: 0,
            lsp_connected: 0,
            lsp_disabled: true,
            mcp_connected: 0,
            mcp_errors: 0,
        }
    }

    pub fn render(&self, f: &mut Frame, area: Rect, theme: &Theme, click_registry: &mut crate::tui::app::click_registry::ClickRegistry) {
        // Sidebar uses background_panel (slightly brighter than the page
        // background) to create a natural color separation between the
        // main area (darker) and sidebar (lighter) — no ASCII border needed.
        let block = Block::default()
            .borders(Borders::LEFT)
            .border_style(Style::default().fg(theme.border))
            .style(Style::default().bg(theme.background_panel));
        f.render_widget(block, area);

        let inner = Rect {
            x: area.x + 1,
            y: area.y + 1,
            width: area.width.saturating_sub(2),
            height: area.height.saturating_sub(2),
        };

        let mut lines: Vec<Line> = Vec::new();

        // 会话标题 (truncate by chars, not bytes — safe for CJK)
        let title_chars: Vec<char> = self.title.chars().collect();
        let title = if title_chars.len() > 38 {
            let head: String = title_chars.into_iter().take(35).collect();
            format!("{}...", head)
        } else {
            self.title.clone()
        };
        lines.push(Line::from(Span::styled(
            title,
            Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(Span::styled(
            self.workspace.clone(),
            Style::default().fg(theme.text_muted),
        )));
        lines.push(Line::from(""));

        // Context 信息（对应 sidebar/context.tsx）
        // 原版只用文字,没有进度条。
        lines.push(Line::from(Span::styled(
            crate::t!("tui.sidebar.context").to_string(),
            Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
        )));
        let tokens_display = format_tokens(self.context_tokens);
        lines.push(Line::from(Span::styled(
            crate::t!("tui.sidebar.tokens", count = tokens_display.as_str()).to_string(),
            Style::default().fg(theme.text_muted),
        )));
        if self.context_limit > 0 {
            let percent = ((self.context_tokens as f64 / self.context_limit as f64) * 100.0) as u32;
            let percent = percent.min(100);
            let pct_str = percent.to_string();
            lines.push(Line::from(Span::styled(
                crate::t!("tui.sidebar.percent_used", percent = pct_str.as_str()).to_string(),
                Style::default().fg(theme.text_muted),
            )));
        }
        if self.context_cost > 0.0 {
            let cost_str = format!("{:.4}", self.context_cost);
            lines.push(Line::from(Span::styled(
                crate::t!("tui.sidebar.spent", amount = cost_str.as_str()).to_string(),
                Style::default().fg(theme.text_muted),
            )));
        }
        // Step + tool call counters
        let steps_str = self.step_count.to_string();
        let tools_str = self.tool_call_count.to_string();
        lines.push(Line::from(Span::styled(
            crate::t!("tui.sidebar.steps_tools", steps = steps_str.as_str(), tools = tools_str.as_str()).to_string(),
            Style::default().fg(theme.text_muted),
        )));
        lines.push(Line::from(""));

        // MCP（对应 sidebar/mcp.tsx）
        if self.mcp_connected > 0 || self.mcp_errors > 0 {
            lines.push(Line::from(Span::styled("MCP", Style::default().fg(theme.text).add_modifier(Modifier::BOLD))));
            if self.mcp_connected > 0 {
                lines.push(Line::from(vec![
                    Span::styled("• ", Style::default().fg(theme.success)),
                    Span::styled(format!("{} connected", self.mcp_connected), Style::default().fg(theme.text_muted)),
                ]));
            }
            if self.mcp_errors > 0 {
                lines.push(Line::from(vec![
                    Span::styled("• ", Style::default().fg(theme.error)),
                    Span::styled(format!("{} error{}", self.mcp_errors, if self.mcp_errors > 1 { "s" } else { "" }), Style::default().fg(theme.text_muted)),
                ]));
            }
            lines.push(Line::from(""));
        }

        // LSP（对应 sidebar/lsp.tsx）
        lines.push(Line::from(Span::styled(
            crate::t!("tui.sidebar.lsp").to_string(),
            Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
        )));
        if self.lsp_disabled {
            lines.push(Line::from(Span::styled(
                crate::t!("tui.sidebar.lsp_disabled").to_string(),
                Style::default().fg(theme.text_muted),
            )));
        } else if self.lsp_connected == 0 {
            lines.push(Line::from(Span::styled(
                crate::t!("tui.sidebar.lsp_pending").to_string(),
                Style::default().fg(theme.text_muted),
            )));
        } else {
            lines.push(Line::from(vec![
                Span::styled("• ", Style::default().fg(theme.success)),
                Span::styled(format!("{} active", self.lsp_connected), Style::default().fg(theme.text_muted)),
            ]));
        }
        lines.push(Line::from(""));

        // Todo（对应 sidebar/todo.tsx）— read from the global TODO_LIST
        // that TodoWrite tool updates.
        let todo_list = crate::tools::todowrite::TODO_LIST.lock().ok();
        let todos: Vec<(String, String)> = todo_list
            .as_ref()
            .map(|list| list.iter()
                .map(|t| (t.content.clone(), t.status.clone()))
                .collect())
            .unwrap_or_default();
        let active_todos: Vec<&(String, String)> = todos.iter()
            .filter(|(_, status)| status != "completed")
            .collect();
        if !active_todos.is_empty() {
            lines.push(Line::from(Span::styled(
                crate::t!("tui.sidebar.todo").to_string(),
                Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
            )));
            for (content, status) in &active_todos {
                let (icon, color) = match status.as_str() {
                    "in_progress" => ("▸", theme.warning),
                    "pending" => ("○", theme.text_muted),
                    "cancelled" => ("✗", theme.error),
                    _ => ("•", theme.text_muted),
                };
                let content_chars: Vec<char> = content.chars().collect();
                let c = if content_chars.len() > 36 {
                    let head: String = content_chars.into_iter().take(33).collect();
                    format!("{}...", head)
                } else {
                    content.clone()
                };
                lines.push(Line::from(vec![
                    Span::styled(format!("{} ", icon), Style::default().fg(color)),
                    Span::styled(c, Style::default().fg(color)),
                ]));
            }
            lines.push(Line::from(""));
        }

        // Version line is rendered separately at the bottom of the sidebar
        // (see below) so it stays anchored to the bottom edge.
        let version_line = Line::from(vec![
            Span::styled("● ", Style::default().fg(theme.success)),
            Span::styled("Open", Style::default().fg(theme.text_muted)),
            Span::styled("Code", Style::default().fg(theme.text)),
            Span::styled(format!(" v{}", self.version), Style::default().fg(theme.text_muted)),
        ]);

        // Process info: PID + memory usage, shown above the version line.
        let pid = std::process::id();
        let mem = current_rss().map(|b| format_memory(b)).unwrap_or_else(|| "—".to_string());
        let proc_line = Line::from(vec![
            Span::styled(format!("{} ", mem), Style::default().fg(theme.text_muted)),
            Span::styled(format!("pid:{}", pid), Style::default().fg(theme.text_muted)),
        ]);

        // Top section: title + context + MCP + LSP + todos.
        let para = Paragraph::new(lines);
        f.render_widget(para, inner);

        // Bottom section: process info + version, anchored to bottom.
        if inner.height >= 3 {
            // Process info on the second-to-last line.
            let proc_area = Rect {
                x: inner.x,
                y: inner.y + inner.height.saturating_sub(2),
                width: inner.width,
                height: 1,
            };
            f.render_widget(Paragraph::new(proc_line), proc_area);
            // Version on the last line.
            let ver_area = Rect {
                x: inner.x,
                y: inner.y + inner.height.saturating_sub(1),
                width: inner.width,
                height: 1,
            };
            f.render_widget(Paragraph::new(version_line), ver_area);
            // Click on the version line → cycle theme.
            click_registry.register(ver_area, "theme:cycle", Some("Click to switch theme".to_string()));
        } else if inner.height >= 1 {
            let bottom_area = Rect {
                x: inner.x,
                y: inner.y + inner.height.saturating_sub(1),
                width: inner.width,
                height: 1,
            };
            f.render_widget(Paragraph::new(version_line), bottom_area);
        }
    }
}

impl Default for Sidebar {
    fn default() -> Self {
        Self::new()
    }
}

/// Human-readable token count (e.g. 1234 → "1.2k", 1000000 → "1.0M").
fn format_tokens(n: u64) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}k", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}

/// Current process resident set size (RSS) in bytes, read from /proc on Linux.
fn current_rss() -> Option<u64> {
    #[cfg(target_os = "linux")]
    {
        let status = std::fs::read_to_string("/proc/self/status").ok()?;
        for line in status.lines() {
            if let Some(rest) = line.strip_prefix("VmRSS:") {
                let kb: u64 = rest.trim().split_whitespace().next()?.parse().ok()?;
                return Some(kb * 1024);
            }
        }
        None
    }
    #[cfg(not(target_os = "linux"))]
    {
        None
    }
}

/// Format a byte count as a human-readable memory string (e.g. "688.9MB").
fn format_memory(bytes: u64) -> String {
    if bytes >= 1024 * 1024 * 1024 {
        format!("{:.1}GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    } else if bytes >= 1024 * 1024 {
        format!("{:.1}MB", bytes as f64 / (1024.0 * 1024.0))
    } else if bytes >= 1024 {
        format!("{:.1}KB", bytes as f64 / 1024.0)
    } else {
        format!("{}B", bytes)
    }
}
