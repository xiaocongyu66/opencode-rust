//! DialogSessionRename — prompt dialog to rename a session.
//! Ported from dialog-session-rename.tsx

use ratatui::layout::{Constraint, Direction, Layout, Rect, Alignment};
use ratatui::style::{Style, Modifier};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Clear, Paragraph, Wrap};
use ratatui::Frame;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crate::tui::theme::Theme;

/// DialogSessionRename — text input dialog for renaming a session.
pub struct DialogSessionRename {
    pub session_id: String,
    pub initial_title: String,
    pub value: String,
    pub visible: bool,
}

#[derive(Debug, Clone)]
pub enum RenameResult {
    None,
    Confirm(String),
    Cancel,
}

impl DialogSessionRename {
    pub fn new(session_id: impl Into<String>, initial_title: impl Into<String>) -> Self {
        let title = initial_title.into();
        Self {
            session_id: session_id.into(),
            initial_title: title.clone(),
            value: title,
            visible: true,
        }
    }

    pub fn close(&mut self) {
        self.visible = false;
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> RenameResult {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return RenameResult::Cancel;
        }
        match key.code {
            KeyCode::Enter => RenameResult::Confirm(self.value.clone()),
            KeyCode::Esc => RenameResult::Cancel,
            KeyCode::Backspace => {
                self.value.pop();
                RenameResult::None
            }
            KeyCode::Char(c) => {
                if !c.is_control() {
                    self.value.push(c);
                }
                RenameResult::None
            }
            _ => RenameResult::None,
        }
    }

    pub fn render(&self, f: &mut Frame, area: Rect, theme: &Theme) {
        if !self.visible {
            return;
        }

        f.render_widget(Clear, area);
        let backdrop = Block::default().style(Style::default().bg(ratatui::style::Color::Rgb(0, 0, 0)));
        f.render_widget(backdrop, area);

        let dialog_width = 60;
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
                Constraint::Length(1),
                Constraint::Min(1),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .split(inner);

        let title_line = Line::from(vec![
            Span::styled("Rename Session", Style::default().fg(theme.text).add_modifier(Modifier::BOLD)),
            Span::raw("  "),
            Span::styled("esc", Style::default().fg(theme.text_muted)),
        ]);
        f.render_widget(Paragraph::new(title_line), chunks[0]);

        let input_line = if self.value.is_empty() {
            Line::from(Span::styled("Session title", Style::default().fg(theme.text_muted)))
        } else {
            Line::from(vec![
                Span::styled("> ", Style::default().fg(theme.accent)),
                Span::styled(self.value.clone(), Style::default().fg(theme.text)),
                Span::styled("▎", Style::default().fg(theme.accent)),
            ])
        };
        f.render_widget(Paragraph::new(input_line), chunks[2]);

        let hint = Paragraph::new(Line::from(Span::styled(
            "enter to confirm  ·  esc to cancel",
            Style::default().fg(theme.text_muted),
        )));
        f.render_widget(hint, chunks[4]);

        let btn_area = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(0), Constraint::Length(8)])
            .split(chunks[6])[1];

        let ok_block = Block::default().style(Style::default().bg(theme.primary));
        f.render_widget(ok_block, btn_area);
        f.render_widget(
            Paragraph::new(Line::from(Span::styled("  save  ", Style::default().fg(theme.text)))).alignment(Alignment::Center),
            btn_area,
        );
    }
}

fn centered_rect(dialog_width: u16, area: Rect) -> Rect {
    let width = std::cmp::min(dialog_width, area.width.saturating_sub(2));
    let height = 7;
    let top = area.height / 3;
    let left = (area.width.saturating_sub(width)) / 2;
    Rect {
        x: area.x + left,
        y: area.y + top,
        width,
        height,
    }
}
