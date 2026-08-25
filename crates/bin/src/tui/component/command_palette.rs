//! Command palette — fuzzy search commands.
//! Ported from tui/src/component/command-palette.tsx

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::Rect;
use ratatui::style::{Style, Modifier};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph};
use ratatui::Frame;
use crate::tui::theme::Theme;

pub struct CommandEntry {
    pub title: String,
    pub description: String,
    pub shortcut: String,
    pub action: String,
}

pub struct CommandPalette {
    pub entries: Vec<CommandEntry>,
    pub filter: String,
    pub selected: usize,
    pub visible: bool,
}

impl CommandPalette {
    pub fn new() -> Self {
        let entries = vec![
            CommandEntry {
                title: "New Session".to_string(),
                description: "Start a new chat session".to_string(),
                shortcut: "Ctrl+N".to_string(),
                action: "new_session".to_string(),
            },
            CommandEntry {
                title: "Switch Agent".to_string(),
                description: "Change the active agent (build/plan)".to_string(),
                shortcut: "Tab".to_string(),
                action: "switch_agent".to_string(),
            },
            CommandEntry {
                title: "Switch Model".to_string(),
                description: "Change the LLM model".to_string(),
                shortcut: "Ctrl+M".to_string(),
                action: "switch_model".to_string(),
            },
            CommandEntry {
                title: "Switch Theme".to_string(),
                description: "Change the color theme".to_string(),
                shortcut: "Ctrl+T".to_string(),
                action: "switch_theme".to_string(),
            },
            CommandEntry {
                title: "List Sessions".to_string(),
                description: "Browse and resume past sessions".to_string(),
                shortcut: "Ctrl+L".to_string(),
                action: "list_sessions".to_string(),
            },
            CommandEntry {
                title: "Toggle Help".to_string(),
                description: "Show keybindings".to_string(),
                shortcut: "?".to_string(),
                action: "help".to_string(),
            },
            CommandEntry {
                title: "Quit".to_string(),
                description: "Exit rsopencode".to_string(),
                shortcut: "q".to_string(),
                action: "quit".to_string(),
            },
        ];

        Self {
            entries,
            filter: String::new(),
            selected: 0,
            visible: false,
        }
    }

    pub fn toggle(&mut self) {
        self.visible = !self.visible;
        if self.visible {
            self.filter.clear();
            self.selected = 0;
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Option<String> {
        match key.code {
            KeyCode::Esc => {
                self.toggle();
                None
            }
            KeyCode::Up => {
                if self.selected > 0 { self.selected -= 1; }
                None
            }
            KeyCode::Down => {
                let max = self.filtered().len().saturating_sub(1);
                if self.selected < max { self.selected += 1; }
                None
            }
            KeyCode::Enter => {
                let action = self.filtered().get(self.selected).map(|e| e.action.clone());
                self.visible = false;
                action
            }
            KeyCode::Backspace => {
                self.filter.pop();
                self.selected = 0;
                None
            }
            KeyCode::Char(c) => {
                self.filter.push(c);
                self.selected = 0;
                None
            }
            _ => None,
        }
    }

    fn filtered(&self) -> Vec<&CommandEntry> {
        if self.filter.is_empty() {
            self.entries.iter().collect()
        } else {
            let filter = self.filter.to_lowercase();
            self.entries.iter()
                .filter(|e| {
                    e.title.to_lowercase().contains(&filter) ||
                    e.description.to_lowercase().contains(&filter)
                })
                .collect()
        }
    }

    pub fn render(&self, f: &mut Frame, area: Rect, theme: &Theme) {
        if !self.visible { return; }

        let popup_width = std::cmp::min(70, area.width.saturating_sub(4));
        let popup_height = std::cmp::min(20, area.height.saturating_sub(4));
        let popup_area = Rect {
            x: area.x + (area.width - popup_width) / 2,
            y: area.y + 2,
            width: popup_width,
            height: popup_height,
        };

        f.render_widget(Clear, popup_area);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border_active))
            .title(Span::styled(
                " Commands ",
                Style::default().fg(theme.primary).add_modifier(Modifier::BOLD),
            ))
            .style(Style::default().bg(theme.background_panel));

        let inner = {
            let mut a = popup_area;
            a.x += 1; a.y += 1;
            a.width = a.width.saturating_sub(2);
            a.height = a.height.saturating_sub(2);
            a
        };

        f.render_widget(block, popup_area);

        // Filter input
        let filter_line = if self.filter.is_empty() {
            Line::from(Span::styled(
                "Type to search commands...",
                Style::default().fg(theme.text_muted),
            ))
        } else {
            Line::from(Span::styled(
                format!("> {}", self.filter),
                Style::default().fg(theme.accent),
            ))
        };
        f.render_widget(Paragraph::new(filter_line), Rect {
            x: inner.x, y: inner.y, width: inner.width, height: 1,
        });

        // Results
        let results_area = Rect {
            x: inner.x, y: inner.y + 1, width: inner.width, height: inner.height.saturating_sub(1),
        };

        let entries = self.filtered();
        let items: Vec<ListItem> = entries.iter().enumerate().map(|(i, e)| {
            let style = if i == self.selected {
                Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.text)
            };
            let shortcut_style = Style::default().fg(theme.text_muted);
            ListItem::new(vec![
                Line::from(vec![
                    Span::styled(e.title.clone(), style),
                    Span::raw("  "),
                    Span::styled(e.shortcut.clone(), shortcut_style),
                ]),
                Line::from(Span::styled(
                    format!("  {}", e.description),
                    Style::default().fg(theme.text_muted),
                )),
                Line::from(""),
            ])
        }).collect();

        f.render_widget(List::new(items), results_area);
    }
}

impl Default for CommandPalette {
    fn default() -> Self {
        Self::new()
    }
}
