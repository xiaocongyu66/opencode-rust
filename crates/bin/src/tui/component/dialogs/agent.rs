//! DialogAgent — agent selector.
//! Ported from dialog-agent.tsx

use ratatui::layout::{Constraint, Direction, Layout, Rect, Alignment};
use ratatui::style::{Style, Modifier};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Clear, List, ListItem, Paragraph, Wrap};
use ratatui::Frame;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crate::tui::theme::Theme;
use crate::tui::component::dialog::{DialogSize, DialogOption};

/// Agent info
#[derive(Clone)]
pub struct AgentInfo {
    pub name: String,
    pub description: String,
    pub native: bool,
}

/// DialogAgent — select dialog for choosing an agent.
pub struct DialogAgent {
    pub agents: Vec<AgentInfo>,
    pub current: Option<String>,
    pub selected: usize,
    pub visible: bool,
}

#[derive(Debug, Clone)]
pub enum AgentResult {
    None,
    Select(String),
    Close,
}

impl DialogAgent {
    pub fn new(agents: Vec<AgentInfo>) -> Self {
        Self {
            agents,
            current: None,
            selected: 0,
            visible: true,
        }
    }

    pub fn close(&mut self) {
        self.visible = false;
    }

    fn build_options(&self) -> Vec<DialogOption> {
        self.agents
            .iter()
            .map(|a| {
                let desc = if a.native { "native" } else { a.description.as_str() };
                DialogOption::new(a.name.clone(), a.name.clone())
                    .with_description(desc)
            })
            .collect()
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> AgentResult {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return AgentResult::Close;
        }
        let opts = self.build_options();
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                if !opts.is_empty() {
                    if self.selected == 0 {
                        self.selected = opts.len() - 1;
                    } else {
                        self.selected -= 1;
                    }
                }
                AgentResult::None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if !opts.is_empty() {
                    let max = opts.len().saturating_sub(1);
                    if self.selected >= max {
                        self.selected = 0;
                    } else {
                        self.selected += 1;
                    }
                }
                AgentResult::None
            }
            KeyCode::Enter => {
                if let Some(opt) = opts.get(self.selected) {
                    AgentResult::Select(opt.value.clone())
                } else {
                    AgentResult::Close
                }
            }
            KeyCode::Esc => AgentResult::Close,
            _ => AgentResult::None,
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
            Span::styled("Select agent", Style::default().fg(theme.text).add_modifier(Modifier::BOLD)),
            Span::raw("  "),
            Span::styled("esc", Style::default().fg(theme.text_muted)),
        ]);
        f.render_widget(Paragraph::new(title_line), chunks[0]);

        let opts = self.build_options();
        let items: Vec<ListItem> = opts
            .iter()
            .enumerate()
            .map(|(i, opt)| {
                let is_selected = i == self.selected;
                let is_current = self.current.as_deref() == Some(&opt.value);
                let style = if is_selected {
                    Style::default().bg(theme.primary).fg(theme.text).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(theme.text)
                };
                let mut spans: Vec<Span> = Vec::new();
                spans.push(Span::styled(if is_selected { "● " } else { "  " }, style));
                spans.push(Span::styled(opt.title.clone(), style));
                if let Some(ref desc) = opt.description {
                    spans.push(Span::styled(format!(" {}", desc), Style::default().fg(theme.text_muted)));
                }
                if is_current && !is_selected {
                    spans.push(Span::styled(" (current)", Style::default().fg(theme.accent)));
                }
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
