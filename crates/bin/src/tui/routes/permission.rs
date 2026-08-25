//! Permission prompt — modal for tool permission requests.
//! Ported from opencode/packages/tui/src/routes/session/permission.tsx (719 lines)
//!
//! Stages:
//!   1. "permission" — show tool info + options (Allow once / Allow always / Reject)
//!   2. "always"     — confirm "always allow" patterns
//!   3. "reject"     — text input for rejection message

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;
use crate::tui::theme::Theme;

// ---------------------------------------------------------------------------
// Permission types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PermissionStage {
    Permission,
    Always,
    Reject,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PermissionReply {
    Once,
    Always,
    Reject,
}

/// The kind of permission being requested — mirrors the TS `permission` field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PermissionKind {
    Edit,
    Read,
    Bash,
    Glob,
    Grep,
    List,
    Task,
    WebFetch,
    WebSearch,
    ExternalDirectory,
    DoomLoop,
    Generic(String),
}

impl PermissionKind {
    pub fn from_str(s: &str) -> Self {
        match s {
            "edit" => Self::Edit,
            "read" => Self::Read,
            "bash" => Self::Bash,
            "glob" => Self::Glob,
            "grep" => Self::Grep,
            "list" => Self::List,
            "task" => Self::Task,
            "webfetch" => Self::WebFetch,
            "websearch" => Self::WebSearch,
            "external_directory" => Self::ExternalDirectory,
            "doom_loop" => Self::DoomLoop,
            other => Self::Generic(other.to_string()),
        }
    }

    pub fn icon(&self) -> &str {
        match self {
            Self::Edit | Self::Read | Self::List => "->",
            Self::Bash | Self::Task => "#",
            Self::Glob | Self::Grep => "*",
            Self::WebFetch => "%",
            Self::WebSearch => "<>",
            Self::ExternalDirectory => "<-",
            Self::DoomLoop => "r",
            Self::Generic(_) => "*",
        }
    }

    pub fn title(&self, input: &PermissionInput) -> String {
        match self {
            Self::Edit => format!("Edit {}", input.filepath),
            Self::Read => format!("Read {}", input.filepath),
            Self::Bash => "Shell command".to_string(),
            Self::Glob => format!("Glob \"{}\"", input.pattern),
            Self::Grep => format!("Grep \"{}\"", input.pattern),
            Self::List => format!("List {}", input.filepath),
            Self::Task => format!("{} Task", titlecase(&input.subagent_type)),
            Self::WebFetch => format!("WebFetch {}", input.url),
            Self::WebSearch => format!("WebSearch \"{}\"", input.query),
            Self::ExternalDirectory => format!("Access external directory {}", input.filepath),
            Self::DoomLoop => "Continue after repeated failures".to_string(),
            Self::Generic(name) => format!("Call tool {}", name),
        }
    }
}

// ---------------------------------------------------------------------------
// Permission input — extracted from tool state
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default)]
pub struct PermissionInput {
    pub filepath: String,
    pub pattern: String,
    pub command: String,
    pub subagent_type: String,
    pub description: String,
    pub url: String,
    pub query: String,
    pub diff: String,
    pub always_patterns: Vec<String>,
}

// ---------------------------------------------------------------------------
// Permission request
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct PermissionRequest {
    pub id: String,
    pub session_id: String,
    pub kind: PermissionKind,
    pub input: PermissionInput,
}

// ---------------------------------------------------------------------------
// Permission result — what handle_key returns
// ---------------------------------------------------------------------------

/// Result of processing a key in the permission prompt.
#[derive(Debug)]
pub enum PermissionResult {
    None,
    /// User selected a reply (Once / Always / Reject).
    Reply(PermissionReply, Option<String>),
}

// ---------------------------------------------------------------------------
// Permission state — holds UI state for the active permission prompt
// ---------------------------------------------------------------------------

pub struct PermissionState {
    pub request: PermissionRequest,
    pub stage: PermissionStage,
    pub selected: usize,
    pub reject_text: String,
    pub reject_cursor: usize,
    pub expanded: bool,
}

/// Options shown in the main permission stage.
const PERMISSION_OPTIONS: &[(&str, &str)] = &[
    ("once", "Allow once"),
    ("always", "Allow always"),
    ("reject", "Reject"),
];

/// Options shown in the "always" confirmation stage.
const ALWAYS_OPTIONS: &[(&str, &str)] = &[
    ("confirm", "Confirm"),
    ("cancel", "Cancel"),
];

impl PermissionState {
    pub fn new(request: PermissionRequest) -> Self {
        Self {
            request,
            stage: PermissionStage::Permission,
            selected: 0,
            reject_text: String::new(),
            reject_cursor: 0,
            expanded: false,
        }
    }

    // -----------------------------------------------------------------------
    // Key handling
    // -----------------------------------------------------------------------

    pub fn handle_key(&mut self, key: KeyEvent) -> PermissionResult {
        match self.stage {
            PermissionStage::Permission => self.handle_permission_key(key),
            PermissionStage::Always => self.handle_always_key(key),
            PermissionStage::Reject => self.handle_reject_key(key),
        }
    }

    fn handle_permission_key(&mut self, key: KeyEvent) -> PermissionResult {
        let count = PERMISSION_OPTIONS.len();
        match key.code {
            KeyCode::Left | KeyCode::Char('h') => {
                if self.selected == 0 {
                    self.selected = count - 1;
                } else {
                    self.selected -= 1;
                }
                PermissionResult::None
            }
            KeyCode::Right | KeyCode::Char('l') => {
                self.selected = (self.selected + 1) % count;
                PermissionResult::None
            }
            KeyCode::Enter => {
                let option = PERMISSION_OPTIONS[self.selected].0;
                match option {
                    "once" => PermissionResult::Reply(PermissionReply::Once, None),
                    "always" => {
                        self.stage = PermissionStage::Always;
                        self.selected = 0;
                        PermissionResult::None
                    }
                    "reject" => {
                        self.stage = PermissionStage::Reject;
                        self.selected = 0;
                        PermissionResult::None
                    }
                    _ => PermissionResult::None,
                }
            }
            KeyCode::Esc => {
                PermissionResult::Reply(PermissionReply::Reject, None)
            }
            _ => PermissionResult::None,
        }
    }

    fn handle_always_key(&mut self, key: KeyEvent) -> PermissionResult {
        let count = ALWAYS_OPTIONS.len();
        match key.code {
            KeyCode::Left | KeyCode::Char('h') => {
                if self.selected == 0 {
                    self.selected = count - 1;
                } else {
                    self.selected -= 1;
                }
                PermissionResult::None
            }
            KeyCode::Right | KeyCode::Char('l') => {
                self.selected = (self.selected + 1) % count;
                PermissionResult::None
            }
            KeyCode::Enter => {
                let option = ALWAYS_OPTIONS[self.selected].0;
                match option {
                    "confirm" => {
                        PermissionResult::Reply(PermissionReply::Always, None)
                    }
                    "cancel" => {
                        self.stage = PermissionStage::Permission;
                        self.selected = 1;
                        PermissionResult::None
                    }
                    _ => PermissionResult::None,
                }
            }
            KeyCode::Esc => {
                self.stage = PermissionStage::Permission;
                self.selected = 1;
                PermissionResult::None
            }
            _ => PermissionResult::None,
        }
    }

    fn handle_reject_key(&mut self, key: KeyEvent) -> PermissionResult {
        match key.code {
            KeyCode::Char(c) => {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    match c {
                        'c' => {
                            self.stage = PermissionStage::Permission;
                            self.selected = 2;
                            PermissionResult::None
                        }
                        _ => PermissionResult::None,
                    }
                } else {
                    self.reject_text.insert(self.reject_cursor, c);
                    self.reject_cursor += c.len_utf8();
                    PermissionResult::None
                }
            }
            KeyCode::Backspace => {
                if self.reject_cursor > 0 {
                    let prev = self.reject_text[..self.reject_cursor]
                        .char_indices()
                        .last()
                        .map(|(i, _)| i)
                        .unwrap_or(0);
                    self.reject_text.replace_range(prev..self.reject_cursor, "");
                    self.reject_cursor = prev;
                }
                PermissionResult::None
            }
            KeyCode::Enter => {
                let msg = self.reject_text.trim().to_string();
                PermissionResult::Reply(
                    PermissionReply::Reject,
                    if msg.is_empty() { None } else { Some(msg) },
                )
            }
            KeyCode::Esc => {
                self.stage = PermissionStage::Permission;
                self.selected = 2;
                PermissionResult::None
            }
            KeyCode::Left => {
                if self.reject_cursor > 0 {
                    let prev = self.reject_text[..self.reject_cursor]
                        .char_indices()
                        .last()
                        .map(|(i, _)| i)
                        .unwrap_or(0);
                    self.reject_cursor = prev;
                }
                PermissionResult::None
            }
            KeyCode::Right => {
                if self.reject_cursor < self.reject_text.len() {
                    let next = self.reject_text[self.reject_cursor..]
                        .char_indices()
                        .nth(1)
                        .map(|(i, _)| self.reject_cursor + i)
                        .unwrap_or(self.reject_text.len());
                    self.reject_cursor = next;
                }
                PermissionResult::None
            }
            _ => PermissionResult::None,
        }
    }

    // -----------------------------------------------------------------------
    // Rendering
    // -----------------------------------------------------------------------

    pub fn render(&self, f: &mut Frame, area: Rect, theme: &Theme) {
        match self.stage {
            PermissionStage::Permission => self.render_permission(f, area, theme),
            PermissionStage::Always => self.render_always(f, area, theme),
            PermissionStage::Reject => self.render_reject(f, area, theme),
        }
    }

    fn render_permission(&self, f: &mut Frame, area: Rect, theme: &Theme) {
        let kind = &self.request.kind;
        let input = &self.request.input;
        let title = kind.title(input);
        let icon = kind.icon();

        let mut lines: Vec<Line> = Vec::new();

        // Header: warning triangle + "Permission required"
        lines.push(Line::from(vec![
            Span::styled("! ", Style::default().fg(theme.warning)),
            Span::styled("Permission required", Style::default().fg(theme.text).add_modifier(Modifier::BOLD)),
        ]));

        // Icon + title
        lines.push(Line::from(vec![
            Span::styled(format!("  {} ", icon), Style::default().fg(theme.text_muted)),
            Span::styled(title, Style::default().fg(theme.text)),
        ]));
        lines.push(Line::from(""));

        // Body — show relevant detail based on kind
        match kind {
            PermissionKind::Bash => {
                if !input.command.is_empty() {
                    lines.push(Line::from(vec![
                        Span::styled("  $ ", Style::default().fg(theme.text_muted)),
                        Span::styled(input.command.clone(), Style::default().fg(theme.text)),
                    ]));
                }
            }
            PermissionKind::Edit => {
                if !input.diff.is_empty() {
                    for line in input.diff.lines().take(8) {
                        let color = if line.starts_with('+') {
                            theme.diff_added
                        } else if line.starts_with('-') {
                            theme.diff_removed
                        } else {
                            theme.text_muted
                        };
                        lines.push(Line::from(Span::styled(
                            format!("  {}", line),
                            Style::default().fg(color),
                        )));
                    }
                    if input.diff.lines().count() > 8 {
                        lines.push(Line::from(Span::styled(
                            "  ...",
                            Style::default().fg(theme.text_muted),
                        )));
                    }
                } else {
                    lines.push(Line::from(Span::styled(
                        "  No diff provided",
                        Style::default().fg(theme.text_muted),
                    )));
                }
            }
            PermissionKind::Task => {
                if !input.description.is_empty() {
                    lines.push(Line::from(vec![
                        Span::styled("  o ", Style::default().fg(theme.text_muted)),
                        Span::styled(input.description.clone(), Style::default().fg(theme.text)),
                    ]));
                }
            }
            PermissionKind::WebFetch => {
                if !input.url.is_empty() {
                    lines.push(Line::from(vec![
                        Span::styled("  URL: ", Style::default().fg(theme.text_muted)),
                        Span::styled(input.url.clone(), Style::default().fg(theme.text)),
                    ]));
                }
            }
            PermissionKind::WebSearch => {
                if !input.query.is_empty() {
                    lines.push(Line::from(vec![
                        Span::styled("  Query: ", Style::default().fg(theme.text_muted)),
                        Span::styled(input.query.clone(), Style::default().fg(theme.text)),
                    ]));
                }
            }
            _ => {}
        }

        lines.push(Line::from(""));

        // Options row
        let mut option_spans: Vec<Span> = Vec::new();
        for (i, (_, label)) in PERMISSION_OPTIONS.iter().enumerate() {
            let is_selected = i == self.selected;
            let (bg, fg) = if is_selected {
                (theme.warning, theme.background)
            } else {
                (theme.background_element, theme.text_muted)
            };
            option_spans.push(Span::styled(
                format!(" {} ", label),
                Style::default().fg(fg).bg(bg),
            ));
            option_spans.push(Span::raw(" "));
        }
        lines.push(Line::from(option_spans));

        // Hints
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("<->", Style::default().fg(theme.text)),
            Span::raw(" "),
            Span::styled("select", Style::default().fg(theme.text_muted)),
            Span::raw("  "),
            Span::styled("enter", Style::default().fg(theme.text)),
            Span::raw(" "),
            Span::styled("confirm", Style::default().fg(theme.text_muted)),
            Span::raw("  "),
            Span::styled("esc", Style::default().fg(theme.text)),
            Span::raw(" "),
            Span::styled("reject", Style::default().fg(theme.text_muted)),
        ]));

        let block = Block::default()
            .borders(Borders::LEFT)
            .border_style(Style::default().fg(theme.warning))
            .style(Style::default().bg(theme.background_panel));

        f.render_widget(block, area);
        f.render_widget(
            Paragraph::new(lines).style(Style::default().bg(theme.background_panel)),
            Rect {
                x: area.x + 1,
                y: area.y,
                width: area.width.saturating_sub(1),
                height: area.height,
            },
        );
    }

    fn render_always(&self, f: &mut Frame, area: Rect, theme: &Theme) {
        let mut lines: Vec<Line> = Vec::new();

        lines.push(Line::from(vec![
            Span::styled("! ", Style::default().fg(theme.warning)),
            Span::styled("Always allow", Style::default().fg(theme.text).add_modifier(Modifier::BOLD)),
        ]));
        lines.push(Line::from(""));

        if self.request.input.always_patterns.len() == 1
            && self.request.input.always_patterns[0] == "*"
        {
            lines.push(Line::from(Span::styled(
                format!(
                    "This will allow {} until OpenCode is restarted.",
                    self.request.kind.title(&self.request.input),
                ),
                Style::default().fg(theme.text_muted),
            )));
        } else {
            lines.push(Line::from(Span::styled(
                "This will allow the following patterns until OpenCode is restarted",
                Style::default().fg(theme.text_muted),
            )));
            for pattern in &self.request.input.always_patterns {
                lines.push(Line::from(vec![
                    Span::raw("  - "),
                    Span::styled(pattern.clone(), Style::default().fg(theme.text)),
                ]));
            }
        }

        lines.push(Line::from(""));

        // Options
        let mut option_spans: Vec<Span> = Vec::new();
        for (i, (_, label)) in ALWAYS_OPTIONS.iter().enumerate() {
            let is_selected = i == self.selected;
            let (bg, fg) = if is_selected {
                (theme.warning, theme.background)
            } else {
                (theme.background_element, theme.text_muted)
            };
            option_spans.push(Span::styled(
                format!(" {} ", label),
                Style::default().fg(fg).bg(bg),
            ));
            option_spans.push(Span::raw(" "));
        }
        lines.push(Line::from(option_spans));

        let block = Block::default()
            .borders(Borders::LEFT)
            .border_style(Style::default().fg(theme.warning))
            .style(Style::default().bg(theme.background_panel));

        f.render_widget(block, area);
        f.render_widget(
            Paragraph::new(lines).style(Style::default().bg(theme.background_panel)),
            Rect {
                x: area.x + 1,
                y: area.y,
                width: area.width.saturating_sub(1),
                height: area.height,
            },
        );
    }

    fn render_reject(&self, f: &mut Frame, area: Rect, theme: &Theme) {
        let mut lines: Vec<Line> = Vec::new();

        lines.push(Line::from(vec![
            Span::styled("! ", Style::default().fg(theme.error)),
            Span::styled("Reject permission", Style::default().fg(theme.text).add_modifier(Modifier::BOLD)),
        ]));
        lines.push(Line::from(Span::styled(
            "Tell OpenCode what to do differently",
            Style::default().fg(theme.text_muted),
        )));
        lines.push(Line::from(""));

        // Textarea
        let display = if self.reject_text.is_empty() {
            "Type rejection message...".to_string()
        } else {
            self.reject_text.clone()
        };
        let style = if self.reject_text.is_empty() {
            Style::default().fg(theme.text_muted)
        } else {
            Style::default().fg(theme.text)
        };
        lines.push(Line::from(Span::styled(format!("  > {}", display), style)));

        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("enter", Style::default().fg(theme.text)),
            Span::raw(" "),
            Span::styled("confirm", Style::default().fg(theme.text_muted)),
            Span::raw("  "),
            Span::styled("esc", Style::default().fg(theme.text)),
            Span::raw(" "),
            Span::styled("cancel", Style::default().fg(theme.text_muted)),
        ]));

        let block = Block::default()
            .borders(Borders::LEFT)
            .border_style(Style::default().fg(theme.error))
            .style(Style::default().bg(theme.background_panel));

        f.render_widget(block, area);
        f.render_widget(
            Paragraph::new(lines).style(Style::default().bg(theme.background_panel)),
            Rect {
                x: area.x + 1,
                y: area.y,
                width: area.width.saturating_sub(1),
                height: area.height,
            },
        );

        // Cursor position
        let cursor_x = area.x + 4 + self.reject_cursor.min(display.len()) as u16;
        let cursor_y = area.y + 3;
        f.set_cursor_position((cursor_x, cursor_y));
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn titlecase(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}
