//! DialogThemeList — theme picker with live preview.
//! Ported from dialog-theme-list.tsx

use ratatui::layout::{Constraint, Direction, Layout, Rect, Alignment};
use ratatui::style::{Style, Modifier};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Clear, List, ListItem, Paragraph};
use ratatui::Frame;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crate::tui::theme::Theme;
use crate::tui::component::dialog::DialogSize;

pub struct DialogThemeList {
    pub themes: Vec<String>,
    pub initial: String,
    pub current: String,
    pub selected: usize,
    pub visible: bool,
}

#[derive(Debug, Clone)]
pub enum ThemeListResult {
    None,
    Preview(String),
    Select(String),
    Close,
}

impl DialogThemeList {
    pub fn new(themes: Vec<String>, current: String) -> Self {
        let initial = current.clone();
        Self {
            themes,
            initial,
            current,
            selected: 0,
            visible: true,
        }
    }

    pub fn close(&mut self) {
        self.visible = false;
    }

    fn sorted_themes(&self) -> Vec<String> {
        let mut list = self.themes.clone();
        list.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));
        list
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> ThemeListResult {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return ThemeListResult::Close;
        }
        let themes = self.sorted_themes();
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                if !themes.is_empty() {
                    if self.selected == 0 {
                        self.selected = themes.len() - 1;
                    } else {
                        self.selected -= 1;
                    }
                    let t = &themes[self.selected];
                    self.current = t.clone();
                    return ThemeListResult::Preview(t.clone());
                }
                ThemeListResult::None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if !themes.is_empty() {
                    let max = themes.len().saturating_sub(1);
                    if self.selected >= max {
                        self.selected = 0;
                    } else {
                        self.selected += 1;
                    }
                    let t = &themes[self.selected];
                    self.current = t.clone();
                    return ThemeListResult::Preview(t.clone());
                }
                ThemeListResult::None
            }
            KeyCode::Enter => {
                if let Some(t) = themes.get(self.selected) {
                    ThemeListResult::Select(t.clone())
                } else {
                    ThemeListResult::Close
                }
            }
            KeyCode::Esc => ThemeListResult::Close,
            _ => ThemeListResult::None,
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
            Span::styled("Themes", Style::default().fg(theme.text).add_modifier(Modifier::BOLD)),
            Span::raw("  "),
            Span::styled("esc", Style::default().fg(theme.text_muted)),
        ]);
        f.render_widget(Paragraph::new(title_line), chunks[0]);

        let themes = self.sorted_themes();
        let items: Vec<ListItem> = themes
            .iter()
            .enumerate()
            .map(|(i, t)| {
                let is_selected = i == self.selected;
                let is_initial = *t == self.initial;
                let style = if is_selected {
                    Style::default().bg(theme.primary).fg(theme.text).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(theme.text)
                };
                let mut spans: Vec<Span> = Vec::new();
                spans.push(Span::styled(if is_selected { "● " } else { "  " }, style));
                spans.push(Span::styled(t.clone(), style));
                if is_initial && !is_selected {
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
