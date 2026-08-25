//! DialogStash — stash entry picker with delete confirmation.
//! Ported from dialog-stash.tsx

use ratatui::layout::{Constraint, Direction, Layout, Rect, Alignment};
use ratatui::style::{Style, Modifier};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Clear, List, ListItem, Paragraph};
use ratatui::Frame;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crate::tui::theme::Theme;
use crate::tui::component::dialog::DialogSize;

#[derive(Clone)]
pub struct StashEntry {
    pub input: String,
    pub timestamp: u64,
}

fn relative_time(timestamp: u64, now: u64) -> String {
    let diff = now.saturating_sub(timestamp);
    let secs = diff / 1000;
    let mins = secs / 60;
    let hours = mins / 60;
    let days = hours / 24;
    if secs < 60 {
        "just now".to_string()
    } else if mins < 60 {
        format!("{}m ago", mins)
    } else if hours < 24 {
        format!("{}h ago", hours)
    } else if days < 7 {
        format!("{}d ago", days)
    } else {
        format!("{} days ago", days)
    }
}

fn stash_preview(input: &str, max_len: usize) -> String {
    let first_line = input.lines().next().unwrap_or("").trim();
    if first_line.len() > max_len {
        format!("{}...", &first_line[..max_len.saturating_sub(3)])
    } else {
        first_line.to_string()
    }
}

pub struct DialogStash {
    pub entries: Vec<StashEntry>,
    pub to_delete: Option<usize>,
    pub removed: Vec<usize>,
    pub selected: usize,
    pub visible: bool,
}

#[derive(Debug, Clone)]
pub enum StashResult {
    None,
    Select(StashEntry),
    Delete(usize),
    Close,
}

impl DialogStash {
    pub fn new(entries: Vec<StashEntry>) -> Self {
        Self {
            entries,
            to_delete: None,
            removed: vec![],
            selected: 0,
            visible: true,
        }
    }

    pub fn close(&mut self) {
        self.visible = false;
    }

    fn visible_indices(&self) -> Vec<usize> {
        let mut indices: Vec<usize> = (0..self.entries.len())
            .filter(|i| !self.removed.contains(i))
            .collect();
        indices.reverse();
        indices
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> StashResult {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return StashResult::Close;
        }
        let indices = self.visible_indices();
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                if !indices.is_empty() {
                    if self.selected == 0 {
                        self.selected = indices.len() - 1;
                    } else {
                        self.selected -= 1;
                    }
                    self.to_delete = None;
                }
                StashResult::None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if !indices.is_empty() {
                    let max = indices.len().saturating_sub(1);
                    if self.selected >= max {
                        self.selected = 0;
                    } else {
                        self.selected += 1;
                    }
                    self.to_delete = None;
                }
                StashResult::None
            }
            KeyCode::Enter => {
                if let Some(&real_idx) = indices.get(self.selected) {
                    if let Some(entry) = self.entries.get(real_idx).cloned() {
                        return StashResult::Select(entry);
                    }
                }
                StashResult::Close
            }
            KeyCode::Esc => StashResult::Close,
            KeyCode::Char('d') => {
                if let Some(&real_idx) = indices.get(self.selected) {
                    if self.to_delete == Some(real_idx) {
                        self.to_delete = None;
                        return StashResult::Delete(real_idx);
                    } else {
                        self.to_delete = Some(real_idx);
                    }
                }
                StashResult::None
            }
            _ => StashResult::None,
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
            Span::styled("Stash", Style::default().fg(theme.text).add_modifier(Modifier::BOLD)),
            Span::raw("  "),
            Span::styled("esc", Style::default().fg(theme.text_muted)),
        ]);
        f.render_widget(Paragraph::new(title_line), chunks[0]);

        let indices = self.visible_indices();
        if indices.is_empty() {
            let empty = Paragraph::new(Line::from(Span::styled(
                "No stashed prompts",
                Style::default().fg(theme.text_muted),
            )));
            f.render_widget(empty, chunks[2]);
            return;
        }

        let now = 0u64;
        let items: Vec<ListItem> = indices
            .iter()
            .enumerate()
            .map(|(display_i, &real_idx)| {
                let is_selected = display_i == self.selected;
                let is_deleting = self.to_delete == Some(real_idx);
                let style = if is_deleting {
                    Style::default().bg(theme.error).fg(theme.text).add_modifier(Modifier::BOLD)
                } else if is_selected {
                    Style::default().bg(theme.primary).fg(theme.text).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(theme.text)
                };
                let entry = &self.entries[real_idx];
                let title = if is_deleting {
                    "Press d again to confirm".to_string()
                } else {
                    stash_preview(&entry.input, 50)
                };
                let line_count = entry.input.lines().count();
                let mut spans: Vec<Span> = Vec::new();
                spans.push(Span::styled(if is_selected { "● " } else { "  " }, style));
                spans.push(Span::styled(title, style));
                spans.push(Span::styled(format!("  {}", relative_time(entry.timestamp, now)), Style::default().fg(theme.text_muted)));
                if line_count > 1 {
                    spans.push(Span::styled(format!("  ~{} lines", line_count), Style::default().fg(theme.text_muted)));
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
