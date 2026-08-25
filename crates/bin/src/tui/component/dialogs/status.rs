//! DialogStatus — status panel for MCP/LSP/Formatters/Plugins.
//! Ported from dialog-status.tsx

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Style, Modifier};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Clear, Paragraph, Wrap};
use ratatui::Frame;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crate::tui::theme::Theme;
use crate::tui::component::dialog::DialogSize;

#[derive(Clone, Debug)]
pub enum McpStatus {
    Connected,
    Failed(Option<String>),
    Disabled,
    NeedsAuth,
    NeedsClientRegistration(Option<String>),
}

#[derive(Clone)]
pub struct McpInfo {
    pub name: String,
    pub status: McpStatus,
}

#[derive(Clone, Debug)]
pub enum LspStatus {
    Connected,
    Error,
}

#[derive(Clone)]
pub struct LspInfo {
    pub id: String,
    pub root: String,
    pub status: LspStatus,
}

#[derive(Clone)]
pub struct FormatterInfo {
    pub name: String,
    pub enabled: bool,
}

#[derive(Clone)]
pub struct PluginInfo {
    pub name: String,
    pub version: Option<String>,
}

pub struct DialogStatus {
    pub mcps: Vec<McpInfo>,
    pub lsps: Vec<LspInfo>,
    pub formatters: Vec<FormatterInfo>,
    pub plugins: Vec<PluginInfo>,
    pub visible: bool,
}

impl DialogStatus {
    pub fn new() -> Self {
        Self {
            mcps: vec![],
            lsps: vec![],
            formatters: vec![],
            plugins: vec![],
            visible: true,
        }
    }

    pub fn close(&mut self) {
        self.visible = false;
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return false;
        }
        matches!(key.code, KeyCode::Esc | KeyCode::Enter)
    }

    pub fn render(&self, f: &mut Frame, area: Rect, theme: &Theme) {
        if !self.visible {
            return;
        }

        f.render_widget(Clear, area);
        let backdrop = Block::default().style(Style::default().bg(ratatui::style::Color::Rgb(0, 0, 0)));
        f.render_widget(backdrop, area);

        let dialog_width = std::cmp::min(DialogSize::Large.width(), area.width.saturating_sub(2));
        let popup_area = centered_rect(dialog_width, area);

        f.render_widget(Clear, popup_area);
        let panel = Block::default().style(Style::default().bg(theme.background_panel));
        f.render_widget(panel, popup_area);

        let inner = Rect {
            x: popup_area.x + 1,
            y: popup_area.y + 1,
            width: popup_area.width.saturating_sub(2),
            height: popup_area.height.saturating_sub(2),
        };

        let mut lines: Vec<Line> = Vec::new();

        lines.push(Line::from(vec![
            Span::styled("Status", Style::default().fg(theme.text).add_modifier(Modifier::BOLD)),
            Span::raw("  "),
            Span::styled("esc", Style::default().fg(theme.text_muted)),
        ]));
        lines.push(Line::raw(""));

        if self.mcps.is_empty() {
            lines.push(Line::from(Span::styled("No MCP Servers", Style::default().fg(theme.text))));
        } else {
            lines.push(Line::from(Span::styled(
                format!("{} MCP Servers", self.mcps.len()),
                Style::default().fg(theme.text),
            )));
            for mcp in &self.mcps {
                let (dot_color, status_text) = match &mcp.status {
                    McpStatus::Connected => (theme.success, "Connected".to_string()),
                    McpStatus::Failed(err) => (theme.error, err.clone().unwrap_or_else(|| "failed".to_string())),
                    McpStatus::Disabled => (theme.text_muted, "Disabled in configuration".to_string()),
                    McpStatus::NeedsAuth => (theme.warning, "Needs authentication".to_string()),
                    McpStatus::NeedsClientRegistration(err) => (theme.error, err.clone().unwrap_or_else(|| "needs client registration".to_string())),
                };
                lines.push(Line::from(vec![
                    Span::styled("• ", Style::default().fg(dot_color)),
                    Span::styled(mcp.name.clone(), Style::default().fg(theme.text)),
                    Span::raw(" "),
                    Span::styled(status_text, Style::default().fg(theme.text_muted)),
                ]));
            }
        }

        if !self.lsps.is_empty() {
            lines.push(Line::raw(""));
            lines.push(Line::from(Span::styled(
                format!("{} LSP Servers", self.lsps.len()),
                Style::default().fg(theme.text),
            )));
            for lsp in &self.lsps {
                let dot_color = match lsp.status {
                    LspStatus::Connected => theme.success,
                    LspStatus::Error => theme.error,
                };
                lines.push(Line::from(vec![
                    Span::styled("• ", Style::default().fg(dot_color)),
                    Span::styled(lsp.id.clone(), Style::default().fg(theme.text)),
                    Span::raw(" "),
                    Span::styled(lsp.root.clone(), Style::default().fg(theme.text_muted)),
                ]));
            }
        }

        let enabled_fmts: Vec<&FormatterInfo> = self.formatters.iter().filter(|f| f.enabled).collect();
        lines.push(Line::raw(""));
        if enabled_fmts.is_empty() {
            lines.push(Line::from(Span::styled("No Formatters", Style::default().fg(theme.text))));
        } else {
            lines.push(Line::from(Span::styled(
                format!("{} Formatters", enabled_fmts.len()),
                Style::default().fg(theme.text),
            )));
            for fmt in &enabled_fmts {
                lines.push(Line::from(vec![
                    Span::styled("• ", Style::default().fg(theme.success)),
                    Span::styled(fmt.name.clone(), Style::default().fg(theme.text)),
                ]));
            }
        }

        lines.push(Line::raw(""));
        if self.plugins.is_empty() {
            lines.push(Line::from(Span::styled("No Plugins", Style::default().fg(theme.text))));
        } else {
            lines.push(Line::from(Span::styled(
                format!("{} Plugins", self.plugins.len()),
                Style::default().fg(theme.text),
            )));
            for plugin in &self.plugins {
                let mut spans = vec![
                    Span::styled("• ", Style::default().fg(theme.success)),
                    Span::styled(plugin.name.clone(), Style::default().fg(theme.text)),
                ];
                if let Some(ref ver) = plugin.version {
                    spans.push(Span::styled(format!(" @{}", ver), Style::default().fg(theme.text_muted)));
                }
                lines.push(Line::from(spans));
            }
        }

        f.render_widget(Paragraph::new(lines).wrap(Wrap { trim: true }), inner);
    }
}

impl Default for DialogStatus {
    fn default() -> Self {
        Self::new()
    }
}

fn centered_rect(dialog_width: u16, area: Rect) -> Rect {
    let width = std::cmp::min(dialog_width, area.width.saturating_sub(2));
    let height = area.height.saturating_sub(area.height / 4);
    let top = area.height / 4;
    let left = (area.width.saturating_sub(width)) / 2;
    Rect {
        x: area.x + left,
        y: area.y + top,
        width,
        height,
    }
}
