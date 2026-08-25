//! DialogModel — model selector with favorites, recents, provider sections.
//! Ported from dialog-model.tsx

use ratatui::layout::{Constraint, Direction, Layout, Rect, Alignment};
use ratatui::style::{Style, Modifier};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Clear, List, ListItem, Paragraph, Wrap};
use ratatui::Frame;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crate::tui::theme::Theme;
use crate::tui::component::dialog::{DialogSize, DialogOption};

/// Model info from a provider
#[derive(Clone)]
pub struct ModelInfo {
    pub provider_id: String,
    pub provider_name: String,
    pub model_id: String,
    pub model_name: String,
    pub release_date: Option<String>,
    pub deprecated: bool,
    pub free: bool,
    pub favorite: bool,
    pub disabled: bool,
}

/// DialogModel — select dialog for choosing an LLM model.
pub struct DialogModel {
    pub models: Vec<ModelInfo>,
    pub provider_id_filter: Option<String>,
    pub current: Option<String>,
    pub query: String,
    pub selected: usize,
    pub scroll_offset: usize,
    pub visible: bool,
    pub connected: bool,
}

#[derive(Debug, Clone)]
pub enum ModelResult {
    None,
    Select { provider_id: String, model_id: String },
    OpenProvider,
    ToggleFavorite { provider_id: String, model_id: String },
    Close,
}

impl DialogModel {
    pub fn new(models: Vec<ModelInfo>) -> Self {
        Self {
            models,
            provider_id_filter: None,
            current: None,
            query: String::new(),
            selected: 0,
            scroll_offset: 0,
            visible: true,
            connected: true,
        }
    }

    pub fn with_provider_filter(mut self, provider_id: impl Into<String>) -> Self {
        self.provider_id_filter = Some(provider_id.into());
        self
    }

    pub fn close(&mut self) {
        self.visible = false;
    }

    fn build_options(&self) -> Vec<DialogOption> {
        let needle = self.query.trim().to_lowercase();
        let mut result: Vec<DialogOption> = Vec::new();

        let mut filtered: Vec<&ModelInfo> = self
            .models
            .iter()
            .filter(|m| !m.deprecated)
            .filter(|m| {
                if let Some(ref pid) = self.provider_id_filter {
                    &m.provider_id == pid
                } else {
                    true
                }
            })
            .filter(|m| {
                needle.is_empty()
                    || m.model_name.to_lowercase().contains(&needle)
                    || m.provider_name.to_lowercase().contains(&needle)
            })
            .collect();

        filtered.sort_by(|a, b| {
            let a_free = a.free;
            let b_free = b.free;
            b_free.cmp(&a_free)
                .then_with(|| b.release_date.cmp(&a.release_date))
                .then_with(|| a.model_name.cmp(&b.model_name))
        });

        let mut current_provider: Option<String> = None;
        for m in &filtered {
            let category = if self.connected { Some(m.provider_name.clone()) } else { None };
            let mut title = m.model_name.clone();
            if m.favorite {
                title.push_str(" (Favorite)");
            }
            let mut opt = DialogOption::new(title, format!("{}/{}", m.provider_id, m.model_id));
            if let Some(cat) = category {
                opt = opt.with_category(cat);
            }
            if m.free {
                opt = opt.with_description("Free".to_string());
            }
            opt.disabled = m.disabled;
            result.push(opt);
            current_provider = Some(m.provider_name.clone());
        }

        result
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> ModelResult {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return ModelResult::Close;
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
                ModelResult::None
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
                ModelResult::None
            }
            KeyCode::PageUp => {
                self.selected = self.selected.saturating_sub(10);
                ModelResult::None
            }
            KeyCode::PageDown => {
                let max = opts.len().saturating_sub(1);
                self.selected = (self.selected + 10).min(max);
                ModelResult::None
            }
            KeyCode::Enter => {
                if let Some(opt) = opts.get(self.selected) {
                    let parts: Vec<&str> = opt.value.splitn(2, '/').collect();
                    if parts.len() == 2 {
                        return ModelResult::Select {
                            provider_id: parts[0].to_string(),
                            model_id: parts[1].to_string(),
                        };
                    }
                }
                ModelResult::Close
            }
            KeyCode::Esc => ModelResult::Close,
            KeyCode::Backspace => {
                self.query.pop();
                self.selected = 0;
                self.scroll_offset = 0;
                ModelResult::None
            }
            KeyCode::Char(c) => {
                if !c.is_control() {
                    self.query.push(c);
                    self.selected = 0;
                    self.scroll_offset = 0;
                }
                ModelResult::None
            }
            _ => ModelResult::None,
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

        let title = if let Some(ref pid) = self.provider_id_filter {
            pid.clone()
        } else {
            "Select model".to_string()
        };
        let title_line = Line::from(vec![
            Span::styled(title, Style::default().fg(theme.text).add_modifier(Modifier::BOLD)),
            Span::raw("  "),
            Span::styled("esc", Style::default().fg(theme.text_muted)),
        ]);
        f.render_widget(Paragraph::new(title_line), chunks[0]);

        let filter_line = if self.query.is_empty() {
            Line::from(Span::styled("Search models", Style::default().fg(theme.text_muted)))
        } else {
            Line::from(Span::styled(format!("> {}", self.query), Style::default().fg(theme.accent)))
        };
        f.render_widget(Paragraph::new(filter_line), chunks[2]);

        let opts = self.build_options();
        if opts.is_empty() {
            let empty = Paragraph::new(Line::from(Span::styled(
                "No models found",
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
                let style = if is_selected {
                    Style::default().bg(theme.primary).fg(theme.text).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(theme.text)
                };
                let mut spans: Vec<Span> = Vec::new();
                if let Some(ref cat) = opt.category {
                    spans.push(Span::styled(format!("{} ", cat), Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)));
                }
                spans.push(Span::styled(if is_selected { "● " } else { "  " }, style));
                spans.push(Span::styled(opt.title.clone(), style));
                if let Some(ref desc) = opt.description {
                    spans.push(Span::styled(format!(" {}", desc), Style::default().fg(theme.text_muted)));
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
