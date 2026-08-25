//! DialogMcp — MCP server list with toggle.
//! Ported from dialog-mcp.tsx

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Style, Modifier};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Clear, List, ListItem, Paragraph};
use ratatui::Frame;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crate::tui::theme::Theme;
use crate::tui::component::dialog::DialogSize;

#[derive(Clone, Debug)]
pub enum McpServerStatus {
    Connected,
    Failed,
    Disabled,
    NeedsAuth,
    NeedsClientRegistration,
}

#[derive(Clone)]
pub struct McpServer {
    pub name: String,
    pub status: McpServerStatus,
    pub enabled: bool,
    pub loading: bool,
}

pub struct DialogMcp {
    pub servers: Vec<McpServer>,
    pub selected: usize,
    pub visible: bool,
}

#[derive(Debug, Clone)]
pub enum McpResult {
    None,
    Toggle(String),
    Close,
}

impl DialogMcp {
    pub fn new(servers: Vec<McpServer>) -> Self {
        Self {
            servers,
            selected: 0,
            visible: true,
        }
    }

    pub fn close(&mut self) {
        self.visible = false;
    }

    fn sorted_servers(&self) -> Vec<&McpServer> {
        let mut sorted: Vec<&McpServer> = self.servers.iter().collect();
        sorted.sort_by(|a, b| a.name.cmp(&b.name));
        sorted
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> McpResult {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return McpResult::Close;
        }
        let servers = self.sorted_servers();
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                if !servers.is_empty() {
                    if self.selected == 0 {
                        self.selected = servers.len() - 1;
                    } else {
                        self.selected -= 1;
                    }
                }
                McpResult::None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if !servers.is_empty() {
                    let max = servers.len().saturating_sub(1);
                    if self.selected >= max {
                        self.selected = 0;
                    } else {
                        self.selected += 1;
                    }
                }
                McpResult::None
            }
            KeyCode::Char('t') => {
                if let Some(s) = servers.get(self.selected) {
                    McpResult::Toggle(s.name.clone())
                } else {
                    McpResult::None
                }
            }
            KeyCode::Esc => McpResult::Close,
            _ => McpResult::None,
        }
    }

    pub fn render(&self, f: &mut Frame, area: Rect, theme: &Theme) {
        if !self.visible {
            return;
        }

        f.render_widget(Clear, area);
        let backdrop = Block::default().style(Style::default().bg(ratatui::style::Color::Rgb(0, 0, 0)));
        f.render_widget(backdrop, area);

        let dialog_width = std::cmp::min(DialogSize::Medium.width(), area.width.saturating_sub(2));
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

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Min(1),
            ])
            .split(inner);

        let title_line = Line::from(vec![
            Span::styled("MCPs", Style::default().fg(theme.text).add_modifier(Modifier::BOLD)),
            Span::raw("  "),
            Span::styled("esc", Style::default().fg(theme.text_muted)),
        ]);
        f.render_widget(Paragraph::new(title_line), chunks[0]);

        let servers = self.sorted_servers();
        let items: Vec<ListItem> = servers
            .iter()
            .enumerate()
            .map(|(i, s)| {
                let is_selected = i == self.selected;
                let style = if is_selected {
                    Style::default().bg(theme.primary).fg(theme.text).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(theme.text)
                };
                let status_text = match s.status {
                    McpServerStatus::Connected => "connected",
                    McpServerStatus::Failed => "failed",
                    McpServerStatus::Disabled => "disabled",
                    McpServerStatus::NeedsAuth => "needs_auth",
                    McpServerStatus::NeedsClientRegistration => "needs_client_registration",
                };
                let footer = if s.loading {
                    "⋯ Loading"
                } else if s.enabled {
                    "✓ Enabled"
                } else {
                    "○ Disabled"
                };
                let mut spans: Vec<Span> = Vec::new();
                spans.push(Span::styled(if is_selected { "● " } else { "  " }, style));
                spans.push(Span::styled(s.name.clone(), style));
                spans.push(Span::styled(format!(" {}", status_text), Style::default().fg(theme.text_muted)));
                spans.push(Span::styled(format!("  {}", footer), Style::default().fg(if s.enabled { theme.success } else { theme.text_muted })));
                ListItem::new(Line::from(spans))
            })
            .collect();

        f.render_widget(List::new(items), chunks[2]);
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
