//! DialogSkill — skill selector.
//! Ported from dialog-skill.tsx

use ratatui::layout::{Constraint, Direction, Layout, Rect, Alignment};
use ratatui::style::{Style, Modifier};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Clear, List, ListItem, Paragraph};
use ratatui::Frame;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crate::tui::theme::Theme;
use crate::tui::component::dialog::DialogSize;

#[derive(Clone)]
pub struct SkillInfo {
    pub name: String,
    pub description: Option<String>,
}

pub struct DialogSkill {
    pub skills: Vec<SkillInfo>,
    pub load_error: Option<String>,
    pub query: String,
    pub selected: usize,
    pub visible: bool,
}

#[derive(Debug, Clone)]
pub enum SkillResult {
    None,
    Select(String),
    Close,
}

impl DialogSkill {
    pub fn new(skills: Vec<SkillInfo>) -> Self {
        Self {
            skills,
            load_error: None,
            query: String::new(),
            selected: 0,
            visible: true,
        }
    }

    pub fn close(&mut self) {
        self.visible = false;
    }

    fn filtered(&self) -> Vec<&SkillInfo> {
        let needle = self.query.trim().to_lowercase();
        self.skills
            .iter()
            .filter(|s| needle.is_empty() || s.name.to_lowercase().contains(&needle))
            .collect()
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> SkillResult {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return SkillResult::Close;
        }
        if self.load_error.is_some() {
            match key.code {
                KeyCode::Esc | KeyCode::Enter => return SkillResult::Close,
                _ => return SkillResult::None,
            }
        }
        let filtered = self.filtered();
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                if !filtered.is_empty() {
                    if self.selected == 0 {
                        self.selected = filtered.len() - 1;
                    } else {
                        self.selected -= 1;
                    }
                }
                SkillResult::None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if !filtered.is_empty() {
                    let max = filtered.len().saturating_sub(1);
                    if self.selected >= max {
                        self.selected = 0;
                    } else {
                        self.selected += 1;
                    }
                }
                SkillResult::None
            }
            KeyCode::Enter => {
                if let Some(s) = filtered.get(self.selected) {
                    SkillResult::Select(s.name.clone())
                } else {
                    SkillResult::Close
                }
            }
            KeyCode::Esc => SkillResult::Close,
            KeyCode::Backspace => {
                self.query.pop();
                self.selected = 0;
                SkillResult::None
            }
            KeyCode::Char(c) => {
                if !c.is_control() {
                    self.query.push(c);
                    self.selected = 0;
                }
                SkillResult::None
            }
            _ => SkillResult::None,
        }
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

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Min(1),
            ])
            .split(inner);

        let title_line = Line::from(vec![
            Span::styled("Skills", Style::default().fg(theme.text).add_modifier(Modifier::BOLD)),
            Span::raw("  "),
            Span::styled("esc", Style::default().fg(theme.text_muted)),
        ]);
        f.render_widget(Paragraph::new(title_line), chunks[0]);

        if let Some(ref err) = self.load_error {
            let err_line = Paragraph::new(vec![
                Line::from(Span::styled("Could not load skills", Style::default().fg(theme.error).add_modifier(Modifier::BOLD))),
                Line::from(Span::styled(err.clone(), Style::default().fg(theme.text_muted))),
            ]);
            f.render_widget(err_line, chunks[3]);
            return;
        }

        let filter_line = if self.query.is_empty() {
            Line::from(Span::styled("Search skills...", Style::default().fg(theme.text_muted)))
        } else {
            Line::from(Span::styled(format!("> {}", self.query), Style::default().fg(theme.accent)))
        };
        f.render_widget(Paragraph::new(filter_line), chunks[2]);

        let filtered = self.filtered();
        if filtered.is_empty() {
            let empty = Paragraph::new(Line::from(Span::styled(
                "No skills found",
                Style::default().fg(theme.text_muted),
            )));
            f.render_widget(empty, chunks[3]);
            return;
        }

        let items: Vec<ListItem> = filtered
            .iter()
            .enumerate()
            .map(|(i, s)| {
                let is_selected = i == self.selected;
                let style = if is_selected {
                    Style::default().bg(theme.primary).fg(theme.text).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(theme.text)
                };
                let mut spans: Vec<Span> = Vec::new();
                spans.push(Span::styled(if is_selected { "● " } else { "  " }, style));
                spans.push(Span::styled(s.name.clone(), style));
                if let Some(ref desc) = s.description {
                    let clean: String = desc.split_whitespace().collect::<Vec<_>>().join(" ");
                    spans.push(Span::styled(format!(" {}", clean), Style::default().fg(theme.text_muted)));
                }
                ListItem::new(Line::from(spans))
            })
            .collect();

        f.render_widget(List::new(items), chunks[3]);
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
