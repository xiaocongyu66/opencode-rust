//! DialogDebug — debug info panel with copy.
//! Ported from dialog-debug.tsx

use ratatui::layout::{Constraint, Direction, Layout, Rect, Alignment};
use ratatui::style::{Style, Modifier};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Clear, Paragraph, Wrap};
use ratatui::Frame;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crate::tui::theme::Theme;
use crate::tui::component::dialog::DialogSize;

#[derive(Clone)]
pub struct DebugEntry {
    pub label: String,
    pub value: String,
}

pub struct DialogDebug {
    pub entries: Vec<DebugEntry>,
    pub copied: bool,
    pub visible: bool,
}

#[derive(Debug, Clone)]
pub enum DebugResult {
    None,
    Copy,
    Close,
}

impl DialogDebug {
    pub fn new(entries: Vec<DebugEntry>) -> Self {
        Self {
            entries,
            copied: false,
            visible: true,
        }
    }

    pub fn close(&mut self) {
        self.visible = false;
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> DebugResult {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return DebugResult::Close;
        }
        match key.code {
            KeyCode::Enter => {
                self.copied = true;
                DebugResult::Copy
            }
            KeyCode::Esc => DebugResult::Close,
            _ => DebugResult::None,
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
                Constraint::Min(1),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .split(inner);

        let title_line = Line::from(vec![
            Span::styled("Debug", Style::default().fg(theme.text).add_modifier(Modifier::BOLD)),
            Span::raw("  "),
            Span::styled("esc", Style::default().fg(theme.text_muted)),
        ]);
        f.render_widget(Paragraph::new(title_line), chunks[0]);

        let mut entry_lines: Vec<Line> = Vec::new();
        for entry in &self.entries {
            entry_lines.push(Line::from(vec![
                Span::styled(format!("{:<10}", entry.label), Style::default().fg(theme.text_muted)),
                Span::raw(" "),
                Span::styled(entry.value.clone(), Style::default().fg(theme.text)),
            ]));
        }
        f.render_widget(
            Paragraph::new(entry_lines).wrap(Wrap { trim: true }),
            chunks[2],
        );

        f.render_widget(
            Paragraph::new(Line::from(Span::styled(
                "Share this when reporting an issue.",
                Style::default().fg(theme.text_muted),
            ))),
            chunks[3],
        );

        let copy_text = if self.copied { "✓ copied" } else { "copy" };
        let copy_color = if self.copied { theme.success } else { theme.text };
        f.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled(copy_text, Style::default().fg(copy_color).add_modifier(Modifier::BOLD)),
                Span::raw(" "),
                Span::styled("enter", Style::default().fg(theme.text_muted)),
            ]))
            .alignment(Alignment::Right),
            chunks[4],
        );
    }
}

fn centered_rect(dialog_width: u16, area: Rect) -> Rect {
    let width = std::cmp::min(dialog_width, area.width.saturating_sub(2));
    let height = 9;
    let top = area.height / 4;
    let left = (area.width.saturating_sub(width)) / 2;
    Rect {
        x: area.x + left,
        y: area.y + top,
        width,
        height,
    }
}
