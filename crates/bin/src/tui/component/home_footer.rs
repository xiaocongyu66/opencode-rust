//! Home footer component — bottom status bar for the home screen.
//! Ported from tui/src/feature-plugins/home/footer.tsx
//!
//! Shows: directory (left, with optional branch), MCP status, LSP status,
//! and version (right).

use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::tui::theme::Theme;

pub struct HomeFooter {
    pub directory: Option<String>,
    pub branch: Option<String>,
    pub mcp_count: usize,
    pub mcp_error: bool,
    pub mcp_has: bool,
    pub version: String,
    pub lsp_count: usize,
    pub lsp_error: bool,
}

impl HomeFooter {
    pub fn new() -> Self {
        Self {
            directory: std::env::current_dir()
                .ok()
                .map(|p| p.to_string_lossy().to_string()),
            branch: None,
            mcp_count: 0,
            mcp_error: false,
            mcp_has: false,
            version: "0.1.0".to_string(),
            lsp_count: 0,
            lsp_error: false,
        }
    }

    fn abbreviate_home(path: &str) -> String {
        if let Some(home) = std::env::var("HOME").ok() {
            if path.starts_with(&home) {
                return format!("~{}", &path[home.len()..]);
            }
        }
        path.to_string()
    }

    pub fn render(&self, f: &mut Frame, area: Rect, theme: &Theme) {
        let mut spans: Vec<Span> = Vec::new();

        if let Some(dir) = &self.directory {
            let mut display = Self::abbreviate_home(dir);
            if let Some(branch) = &self.branch {
                display.push(':');
                display.push_str(branch);
            }
            spans.push(Span::styled(
                display,
                Style::default().fg(theme.text_muted),
            ));
            spans.push(Span::raw("  "));
        }

        if self.mcp_has {
            let icon_color = if self.mcp_error {
                theme.error
            } else if self.mcp_count > 0 {
                theme.success
            } else {
                theme.text_muted
            };
            spans.push(Span::styled("⊙ ", Style::default().fg(icon_color)));
            spans.push(Span::styled(
                format!("{} MCP", self.mcp_count),
                Style::default().fg(theme.text),
            ));
            spans.push(Span::raw("  "));
            spans.push(Span::styled("/status", Style::default().fg(theme.text_muted)));
        }

        if self.lsp_count > 0 {
            if !spans.is_empty() {
                spans.push(Span::raw("  "));
            }
            let lsp_color = if self.lsp_error {
                theme.error
            } else {
                theme.success
            };
            spans.push(Span::styled("● ", Style::default().fg(lsp_color)));
            spans.push(Span::styled(
                format!("{} LSP", self.lsp_count),
                Style::default().fg(theme.text),
            ));
        }

        spans.push(Span::raw(" "));

        spans.push(Span::styled(
            self.version.clone(),
            Style::default().fg(theme.text_muted),
        ));

        let line = Line::from(spans);
        let paragraph = Paragraph::new(line);
        f.render_widget(paragraph, area);
    }
}

impl Default for HomeFooter {
    fn default() -> Self {
        Self::new()
    }
}
