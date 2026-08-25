//! Footer component — 底部状态栏，对应原版 routes/session/footer.tsx
//! 左侧：当前目录  右侧：权限数 / LSP / MCP / /status

use ratatui::layout::Rect;
use ratatui::style::{Style, Modifier};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;
use crate::tui::theme::Theme;

pub struct Footer {
    pub directory: String,
    pub lsp_count: usize,
    pub mcp_count: usize,
    pub mcp_error: bool,
    pub permission_count: usize,
    pub agent: String,
    pub status: String,
    pub version: String,
    pub connected: bool,
}

impl Footer {
    pub fn new() -> Self {
        Self {
            directory: std::env::current_dir()
                .map(|p| {
                    let s = p.to_string_lossy().to_string();
                    if let Some(home) = std::env::var("HOME").ok() {
                        if s.starts_with(&home) {
                            return format!("~{}", &s[home.len()..]);
                        }
                    }
                    s
                })
                .unwrap_or_default(),
            lsp_count: 0,
            mcp_count: 0,
            mcp_error: false,
            permission_count: 0,
            agent: "build".to_string(),
            status: crate::t!("tui.status.idle").to_string(),
            version: "0.1.0".to_string(),
            connected: true,
        }
    }

    pub fn render(&self, f: &mut Frame, area: Rect, theme: &Theme) {
        let mut all: Vec<Span> = vec![
            Span::raw(" "),
            Span::styled(self.directory.clone(), Style::default().fg(theme.text_muted)),
            Span::raw("  "),
        ];

        if self.permission_count > 0 {
            let suffix = if self.permission_count > 1 { "s" } else { "" };
            all.push(Span::styled(
                crate::t!("tui.footer.permission", count = self.permission_count, suffix = suffix).to_string(),
                Style::default().fg(theme.warning),
            ));
        }

        let lsp_color = if self.lsp_count > 0 { theme.success } else { theme.text_muted };
        all.push(Span::styled("• ", Style::default().fg(lsp_color)));
        all.push(Span::styled(
            format!("{}  ", crate::t!("tui.footer.lsp", count = self.lsp_count)),
            Style::default().fg(theme.text),
        ));

        if self.mcp_count > 0 {
            let mcp_color = if self.mcp_error { theme.error } else { theme.success };
            all.push(Span::styled("⊙ ", Style::default().fg(mcp_color)));
            all.push(Span::styled(
                format!("{}  ", crate::t!("tui.footer.mcp", count = self.mcp_count)),
                Style::default().fg(theme.text),
            ));
        }

        all.push(Span::styled(crate::t!("tui.footer.status_cmd").to_string(), Style::default().fg(theme.text_muted)));
        all.push(Span::raw("  "));
        all.push(Span::styled(
            format!("v{}", self.version),
            Style::default().fg(theme.text_muted),
        ));

        let para = Paragraph::new(Line::from(all)).style(Style::default().bg(theme.background));
        f.render_widget(para, area);
    }
}
