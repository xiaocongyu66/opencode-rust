//! DialogProvider — provider connection selector.
//! Ported from dialog-provider.tsx

use ratatui::layout::{Constraint, Direction, Layout, Rect, Alignment};
use ratatui::style::{Style, Modifier};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Clear, List, ListItem, Paragraph};
use ratatui::Frame;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crate::tui::theme::Theme;
use crate::tui::component::dialog::{DialogSize, DialogOption};

const PROVIDER_PRIORITY: &[(&str, u32)] = &[
    ("opencode", 0),
    ("opencode-go", 1),
    ("openai", 2),
    ("github-copilot", 3),
    ("anthropic", 4),
    ("google", 5),
];

const CUSTOM_PROVIDER_OPTION_VALUE: &str = "__opencode_custom_provider__";

#[derive(Clone)]
pub struct ProviderInfo {
    pub id: String,
    pub name: String,
}

#[derive(Clone)]
pub struct ProviderOption {
    pub title: String,
    pub value: String,
    pub description: Option<String>,
    pub category: String,
    pub is_custom: bool,
}

fn provider_priority(id: &str) -> u32 {
    PROVIDER_PRIORITY
        .iter()
        .find(|(k, _)| *k == id)
        .map(|(_, v)| *v)
        .unwrap_or(99)
}

pub fn provider_options(list: &[ProviderInfo]) -> Vec<ProviderOption> {
    let mut sorted: Vec<&ProviderInfo> = list.iter().collect();
    sorted.sort_by(|a, b| {
        provider_priority(&a.id)
            .cmp(&provider_priority(&b.id))
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
            .then_with(|| a.id.cmp(&b.id))
    });

    let mut result: Vec<ProviderOption> = sorted
        .iter()
        .map(|p| {
            let desc = match p.id.as_str() {
                "opencode" => Some("(Recommended)".to_string()),
                "anthropic" => Some("(API key)".to_string()),
                "openai" => Some("(ChatGPT Plus/Pro or API key)".to_string()),
                "opencode-go" => Some("Low cost subscription for everyone".to_string()),
                _ => None,
            };
            let known = PROVIDER_PRIORITY.iter().any(|(k, _)| *k == p.id);
            let category = if known { "Popular" } else { "Providers" };
            ProviderOption {
                title: p.name.clone(),
                value: p.id.clone(),
                description: desc,
                category: category.to_string(),
                is_custom: false,
            }
        })
        .collect();

    result.push(ProviderOption {
        title: "Other".to_string(),
        value: CUSTOM_PROVIDER_OPTION_VALUE.to_string(),
        description: Some("Custom provider".to_string()),
        category: "Providers".to_string(),
        is_custom: true,
    });
    result
}

pub struct DialogProvider {
    pub options: Vec<ProviderOption>,
    pub connected: Vec<String>,
    pub selected: usize,
    pub visible: bool,
}

#[derive(Debug, Clone)]
pub enum ProviderResult {
    None,
    Select(String),
    Close,
}

impl DialogProvider {
    pub fn new(providers: Vec<ProviderInfo>) -> Self {
        Self {
            options: provider_options(&providers),
            connected: vec![],
            selected: 0,
            visible: true,
        }
    }

    pub fn close(&mut self) {
        self.visible = false;
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> ProviderResult {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return ProviderResult::Close;
        }
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                if !self.options.is_empty() {
                    if self.selected == 0 {
                        self.selected = self.options.len() - 1;
                    } else {
                        self.selected -= 1;
                    }
                }
                ProviderResult::None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if !self.options.is_empty() {
                    let max = self.options.len().saturating_sub(1);
                    if self.selected >= max {
                        self.selected = 0;
                    } else {
                        self.selected += 1;
                    }
                }
                ProviderResult::None
            }
            KeyCode::Enter => {
                if let Some(opt) = self.options.get(self.selected) {
                    ProviderResult::Select(opt.value.clone())
                } else {
                    ProviderResult::Close
                }
            }
            KeyCode::Esc => ProviderResult::Close,
            _ => ProviderResult::None,
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
            Span::styled("Connect a provider", Style::default().fg(theme.text).add_modifier(Modifier::BOLD)),
            Span::raw("  "),
            Span::styled("esc", Style::default().fg(theme.text_muted)),
        ]);
        f.render_widget(Paragraph::new(title_line), chunks[0]);

        let items: Vec<ListItem> = self
            .options
            .iter()
            .enumerate()
            .map(|(i, opt)| {
                let is_selected = i == self.selected;
                let is_connected = self.connected.iter().any(|c| c == &opt.value);
                let style = if is_selected {
                    Style::default().bg(theme.primary).fg(theme.text).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(theme.text)
                };
                let mut spans: Vec<Span> = Vec::new();
                if !opt.category.is_empty() {
                    spans.push(Span::styled(format!("{} ", opt.category), Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)));
                }
                spans.push(Span::styled(if is_selected { "● " } else { "  " }, style));
                spans.push(Span::styled(opt.title.clone(), style));
                if let Some(ref desc) = opt.description {
                    spans.push(Span::styled(format!(" {}", desc), Style::default().fg(theme.text_muted)));
                }
                if is_connected {
                    spans.push(Span::styled(" ✓", Style::default().fg(theme.success)));
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
