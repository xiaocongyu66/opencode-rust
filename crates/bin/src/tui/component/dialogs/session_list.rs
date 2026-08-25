//! DialogSessionList — session picker with search, pin, delete, rename.
//! Ported from dialog-session-list.tsx

use ratatui::layout::{Constraint, Direction, Layout, Rect, Alignment};
use ratatui::style::{Style, Modifier};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Clear, List, ListItem, Paragraph, Wrap};
use ratatui::Frame;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crate::tui::theme::Theme;
use crate::tui::component::dialog::{Dialog, DialogSize, DialogOption, DialogResult};

/// Session info shown in the list
#[derive(Clone)]
pub struct SessionInfo {
    pub id: String,
    pub title: String,
    pub updated: u64,
    pub workspace_id: Option<String>,
    pub directory: Option<String>,
    pub path: Option<String>,
    pub pinned: bool,
    pub working: bool,
    pub slot: Option<usize>,
}

/// DialogSessionList — large select dialog listing sessions by recency.
pub struct DialogSessionList {
    pub sessions: Vec<SessionInfo>,
    pub current_session_id: Option<String>,
    pub deleted: std::collections::HashSet<String>,
    pub to_delete: Option<String>,
    pub search: String,
    pub selected: usize,
    pub scroll_offset: usize,
    pub visible: bool,
}

impl DialogSessionList {
    pub fn new(sessions: Vec<SessionInfo>) -> Self {
        Self {
            sessions,
            current_session_id: None,
            deleted: std::collections::HashSet::new(),
            to_delete: None,
            search: String::new(),
            selected: 0,
            scroll_offset: 0,
            visible: true,
        }
    }

    pub fn close(&mut self) {
        self.visible = false;
    }

    fn filtered_sessions(&self) -> Vec<&SessionInfo> {
        let query = self.search.trim().to_lowercase();
        self.sessions
            .iter()
            .filter(|s| !self.deleted.contains(&s.id))
            .filter(|s| query.is_empty() || s.title.to_lowercase().contains(&query))
            .collect()
    }

    fn order_by_recency(&self) -> Vec<usize> {
        let mut indices: Vec<usize> = self
            .sessions
            .iter()
            .enumerate()
            .filter(|(_, s)| s.path.is_none()) // parentID === undefined approximated by no path
            .map(|(i, s)| (i, s.updated))
            .collect();
        indices.sort_by(|a, b| b.1.cmp(&a.1));
        indices.into_iter().map(|(i, _)| i).collect()
    }

    fn build_options(&self) -> Vec<DialogOption> {
        let today = "Today";
        let session_map: std::collections::HashMap<&str, &SessionInfo> =
            self.sessions.iter().map(|s| (s.id.as_str(), s)).collect();

        let pinned: Vec<&SessionInfo> = self
            .sessions
            .iter()
            .filter(|s| s.pinned)
            .collect();

        let order = self.order_by_recency();
        let pinned_set: std::collections::HashSet<&str> =
            pinned.iter().map(|s| s.id.as_str()).collect();

        let remaining: Vec<&SessionInfo> = order
            .iter()
            .filter_map(|&i| self.sessions.get(i))
            .filter(|s| !pinned_set.contains(s.id.as_str()))
            .collect();

        let mut result: Vec<DialogOption> = Vec::new();

        for s in &pinned {
            if let Some(opt) = self.build_option(s, "Pinned", session_map.values().cloned().collect()) {
                result.push(opt);
            }
        }
        for s in &remaining {
            let label = if is_today(s.updated) { today } else { "Earlier" };
            if let Some(opt) = self.build_option(s, label, vec![]) {
                result.push(opt);
            }
        }
        result
    }

    fn build_option(
        &self,
        s: &SessionInfo,
        category: &str,
        _all: Vec<&SessionInfo>,
    ) -> Option<DialogOption> {
        let is_deleting = self.to_delete.as_deref() == Some(&s.id);
        let title = if is_deleting {
            "Press delete again to confirm".to_string()
        } else {
            s.title.clone()
        };
        let mut opt = DialogOption::new(title, s.id.clone())
            .with_category(category);
        if is_deleting {
            opt.disabled = false;
        }
        Some(opt)
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> SessionListResult {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return SessionListResult::Close;
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
                    self.to_delete = None;
                }
                SessionListResult::None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if !opts.is_empty() {
                    let max = opts.len().saturating_sub(1);
                    if self.selected >= max {
                        self.selected = 0;
                    } else {
                        self.selected += 1;
                    }
                    self.to_delete = None;
                }
                SessionListResult::None
            }
            KeyCode::Enter => {
                if let Some(opt) = opts.get(self.selected) {
                    SessionListResult::Select(opt.value.clone())
                } else {
                    SessionListResult::Close
                }
            }
            KeyCode::Esc => SessionListResult::Close,
            KeyCode::Backspace => {
                self.search.pop();
                self.selected = 0;
                self.scroll_offset = 0;
                SessionListResult::None
            }
            KeyCode::Char(c) => {
                if !c.is_control() {
                    self.search.push(c);
                    self.selected = 0;
                    self.scroll_offset = 0;
                }
                SessionListResult::None
            }
            KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::NONE) => {
                // Toggle delete confirmation
                if let Some(opt) = opts.get(self.selected) {
                    if self.to_delete.as_deref() == Some(&opt.value) {
                        SessionListResult::Delete(opt.value.clone())
                    } else {
                        self.to_delete = Some(opt.value.clone());
                        SessionListResult::None
                    }
                } else {
                    SessionListResult::None
                }
            }
            _ => SessionListResult::None,
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
                Constraint::Length(1),
                Constraint::Min(1),
            ])
            .split(inner);

        let title_line = Line::from(vec![
            Span::styled("Sessions", Style::default().fg(theme.text).add_modifier(Modifier::BOLD)),
            Span::raw("  "),
            Span::styled("esc", Style::default().fg(theme.text_muted)),
        ]);
        f.render_widget(Paragraph::new(title_line), chunks[0]);

        let filter_line = if self.search.is_empty() {
            Line::from(Span::styled("Search", Style::default().fg(theme.text_muted)))
        } else {
            Line::from(Span::styled(format!("> {}", self.search), Style::default().fg(theme.accent)))
        };
        f.render_widget(Paragraph::new(filter_line), chunks[2]);

        let opts = self.build_options();
        if opts.is_empty() {
            let empty = Paragraph::new(Line::from(Span::styled(
                "No sessions found",
                Style::default().fg(theme.text_muted),
            )));
            f.render_widget(empty, chunks[4]);
            return;
        }

        let list_area = chunks[4];
        let visible_height = list_area.height as usize;

        let mut scroll = self.scroll_offset;
        if self.selected < scroll {
            scroll = self.selected;
        } else if self.selected >= scroll + visible_height {
            scroll = self.selected.saturating_sub(visible_height.saturating_sub(1));
        }

        let items: Vec<ListItem> = opts
            .iter()
            .enumerate()
            .skip(scroll)
            .take(visible_height)
            .enumerate()
            .map(|(display_i, opt)| {
                let real_index = scroll + display_i;
                let is_selected = real_index == self.selected;
                let is_current = self.current_session_id.as_deref() == Some(&opt.value);
                let is_deleting = self.to_delete.as_deref() == Some(&opt.value);

                let style = if is_deleting {
                    Style::default().bg(theme.error).fg(theme.text).add_modifier(Modifier::BOLD)
                } else if is_selected {
                    Style::default().bg(theme.primary).fg(theme.text).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(theme.text)
                };

                let mut spans: Vec<Span> = Vec::new();
                if let Some(ref cat) = opt.category {
                    spans.push(Span::styled(
                        format!("{} ", cat),
                        Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
                    ));
                }
                spans.push(Span::styled(if is_selected { "● " } else { "  " }, style));
                spans.push(Span::styled(opt.title.clone(), style));
                if is_current && !is_selected {
                    spans.push(Span::styled(" (current)", Style::default().fg(theme.accent)));
                }
                ListItem::new(Line::from(spans))
            })
            .collect();

        f.render_widget(List::new(items), list_area);

        if opts.len() > visible_height {
            let scroll_info = format!(" {}/{} ", self.selected + 1, opts.len());
            let scroll_para = Paragraph::new(Line::from(Span::styled(scroll_info, Style::default().fg(theme.text_muted))))
                .alignment(Alignment::Right);
            let scroll_area = Rect {
                x: list_area.x,
                y: list_area.bottom().saturating_sub(1),
                width: list_area.width,
                height: 1,
            };
            f.render_widget(scroll_para, scroll_area);
        }
    }
}

impl Default for DialogSessionList {
    fn default() -> Self {
        Self::new(vec![])
    }
}

#[derive(Debug, Clone)]
pub enum SessionListResult {
    None,
    Select(String),
    Delete(String),
    Close,
}

fn is_today(_timestamp: u64) -> bool {
    true
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
