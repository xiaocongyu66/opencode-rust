//! Toast notifications — transient messages.
//! Ported from tui/src/ui/toast.tsx
//!
//! Features:
//! - 4 variants (info, success, warning, error) each with a distinct border color.
//! - Left/right split border using `┃` (from `SplitBorder`).
//! - Auto-expiry via duration (default 5s, matching the TS default of 5000ms).
//! - Positioned top-right with padding, max width 60.

use std::time::{Duration, Instant};
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Style, Modifier};
use ratatui::symbols::border;
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;
use crate::tui::theme::Theme;
use super::border::split_border_set;

/// Toast variant — determines the left border color.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastVariant {
    Info,
    Success,
    Warning,
    Error,
}

impl ToastVariant {
    pub fn color(self, theme: &Theme) -> ratatui::style::Color {
        match self {
            ToastVariant::Info => theme.info,
            ToastVariant::Success => theme.success,
            ToastVariant::Warning => theme.warning,
            ToastVariant::Error => theme.error,
        }
    }
}

/// Options for showing a toast — mirrors `ToastOptions` in the TS source.
#[derive(Debug, Clone)]
pub struct ToastOptions {
    pub title: Option<String>,
    pub message: String,
    pub variant: ToastVariant,
    pub duration: Duration,
}

/// Convenience input — duration defaults to 5s when omitted, matching the TS source.
#[derive(Debug, Clone)]
pub struct ToastInput {
    pub title: Option<String>,
    pub message: String,
    pub variant: ToastVariant,
    pub duration: Option<Duration>,
}

impl From<ToastInput> for ToastOptions {
    fn from(input: ToastInput) -> Self {
        ToastOptions {
            title: input.title,
            message: input.message,
            variant: input.variant,
            duration: input.duration.unwrap_or(Duration::from_secs(5)),
        }
    }
}

/// A single toast notification with creation timestamp for expiry tracking.
pub struct Toast {
    pub title: Option<String>,
    pub message: String,
    pub variant: ToastVariant,
    pub created: Instant,
    pub duration: Duration,
}

impl Toast {
    pub fn new(message: impl Into<String>, variant: ToastVariant) -> Self {
        Self {
            title: None,
            message: message.into(),
            variant,
            created: Instant::now(),
            duration: Duration::from_secs(5),
        }
    }

    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }

    pub fn from_options(opts: ToastOptions) -> Self {
        Self {
            title: opts.title,
            message: opts.message,
            variant: opts.variant,
            created: Instant::now(),
            duration: opts.duration,
        }
    }

    pub fn is_expired(&self) -> bool {
        self.created.elapsed() >= self.duration
    }

    /// Render the toast in the top-right corner of `area`.
    ///
    /// Mirrors the TS layout:
    /// - `position: absolute; top: 2; right: 2`
    /// - `maxWidth: min(60, width - 6)`
    /// - `paddingLeft: 2; paddingRight: 2; paddingTop: 1; paddingBottom: 1`
    /// - `backgroundColor: theme.backgroundPanel`
    /// - `borderColor: theme[variant]; border: ["left","right"]`
    /// - `customBorderChars: SplitBorder.customBorderChars` (vertical `┃`)
    pub fn render(&self, f: &mut Frame, area: Rect, theme: &Theme) {
        let color = self.variant.color(theme);

        // maxWidth = min(60, width - 6)
        let max_width = std::cmp::min(60u16, area.width.saturating_sub(6));
        if max_width < 4 {
            return;
        }

        // Wrap message text to fit inside the toast body.
        let inner_width = max_width.saturating_sub(4) as usize; // paddingLeft + paddingRight
        let wrapped = wrap_text(&self.message, inner_width);

        // height = paddingTop(1) + content_lines + paddingBottom(1) + borders
        let content_lines = wrapped.len();
        let title_lines = if self.title.is_some() { 1 } else { 0 };
        let gap = if self.title.is_some() && content_lines > 0 { 1 } else { 0 };
        let toast_height = (1 + title_lines + gap + content_lines + 1) as u16;

        let toast_area = Rect {
            x: area.right().saturating_sub(max_width + 2), // right: 2 margin
            y: area.y + 2,                                   // top: 2
            width: max_width + 2,                            // +2 for left/right borders
            height: toast_height,
        };

        // Clear the background behind the toast.
        f.render_widget(Clear, toast_area);

        // Build the block with split border (left + right) using `┃`.
        let block = Block::default()
            .borders(Borders::LEFT | Borders::RIGHT)
            .border_set(border::Set {
                top_left: "",
                top_right: "",
                bottom_left: "",
                bottom_right: "",
                vertical_left: "┃",
                vertical_right: "┃",
                horizontal_top: " ",
                horizontal_bottom: " ",
            })
            .border_style(Style::default().fg(color))
            .style(Style::default().bg(theme.background_panel))
            .padding(ratatui::widgets::Padding::new(2, 2, 1, 1));

        let mut lines: Vec<Line> = Vec::new();

        // Title (bold)
        if let Some(ref title) = self.title {
            lines.push(Line::from(Span::styled(
                title.clone(),
                Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
            )));
            if content_lines > 0 {
                lines.push(Line::from(""));
            }
        }

        // Message body (word-wrapped)
        for line in &wrapped {
            lines.push(Line::from(Span::styled(
                line.clone(),
                Style::default().fg(theme.text),
            )));
        }

        let paragraph = Paragraph::new(Text::from(lines)).block(block);
        f.render_widget(paragraph, toast_area);

        // Suppress unused warning — split_border_set is re-exported for consumers.
        let _ = split_border_set();
    }
}

/// Simple word-wrap for toast messages.
fn wrap_text(text: &str, max_width: usize) -> Vec<String> {
    if max_width == 0 {
        return vec![text.to_string()];
    }

    let mut result = Vec::new();
    for paragraph in text.split('\n') {
        let mut current = String::new();
        for word in paragraph.split_whitespace() {
            if current.is_empty() {
                current = word.to_string();
            } else if current.len() + 1 + word.len() <= max_width {
                current.push(' ');
                current.push_str(word);
            } else {
                result.push(std::mem::take(&mut current));
                current = word.to_string();
            }
        }
        if !current.is_empty() || result.is_empty() {
            result.push(current);
        }
    }
    result
}

/// Toast manager — holds the current toast and manages expiry.
///
/// Mirrors `init()` in toast.tsx which returns `{ show, error, currentToast }`.
pub struct ToastManager {
    pub current: Option<Toast>,
}

impl ToastManager {
    pub fn new() -> Self {
        Self { current: None }
    }

    /// Show a toast from full options — replaces any current toast, matching the TS `show()` behavior.
    pub fn show_options(&mut self, options: impl Into<ToastOptions>) {
        let opts: ToastOptions = options.into();
        self.current = Some(Toast::from_options(opts));
    }

    /// Show a toast with a message and variant — convenience matching the
    /// TS `toast.show({ message, variant })` pattern. Default duration 5s.
    pub fn show(&mut self, message: impl Into<String>, variant: ToastVariant) {
        self.current = Some(Toast::new(message, variant));
    }

    /// Show a toast from a `ToastInput` (title + message + variant, optional duration).
    pub fn show_input(&mut self, input: ToastInput) {
        let opts: ToastOptions = input.into();
        self.current = Some(Toast::from_options(opts));
    }

    /// Show an error toast from an error-like message.
    /// Mirrors `toast.error(err)` in the TS source.
    pub fn error(&mut self, err: &str) {
        self.current = Some(Toast::new(
            if err.is_empty() {
                "An unknown error has occurred"
            } else {
                err
            },
            ToastVariant::Error,
        ));
    }

    /// Check and expire the current toast. Call on each tick.
    pub fn tick(&mut self) {
        if let Some(ref toast) = self.current {
            if toast.is_expired() {
                self.current = None;
            }
        }
    }

    pub fn render(&self, f: &mut Frame, area: Rect, theme: &Theme) {
        if let Some(ref toast) = self.current {
            toast.render(f, area, theme);
        }
    }
}

impl Default for ToastManager {
    fn default() -> Self {
        Self::new()
    }
}

// Keep the alignment import available for potential future use.
#[allow(dead_code)]
fn _alignment_used() -> Alignment {
    Alignment::Left
}
