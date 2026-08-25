//! Dialog system — modal overlays.
//! Ported from tui/src/ui/dialog.tsx + dialog-select.tsx + dialog-alert.tsx + dialog-confirm.tsx + dialog-help.tsx
//!
//! Features:
//! - Three sizes: medium (60), large (88), xlarge (116) — matching the TS `width()` function.
//! - Semi-transparent overlay backdrop (RGBA(0,0,0,150)) — approximated with Clear + dark bg.
//! - DialogSelect with fuzzy/substring search filter.
//! - DialogAlert (title + message + OK button).
//! - DialogConfirm (title + message + Cancel/Confirm toggle with left/right).
//! - DialogHelp (help text + OK button).
//! - Escape/Ctrl+C to close any dialog.

pub mod impl_;

use std::time::Instant;
use ratatui::layout::{Constraint, Direction, Layout, Rect, Alignment};
use ratatui::style::{Style, Modifier, Color};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap};
use ratatui::Frame;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crate::tui::theme::Theme;

// ---------------------------------------------------------------------------
// Dialog size
// ---------------------------------------------------------------------------

/// Dialog width in columns — matches the TS `width()` function.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogSize {
    Medium,
    Large,
    Xlarge,
}

impl DialogSize {
    pub fn width(self) -> u16 {
        match self {
            DialogSize::Medium => 60,
            DialogSize::Large => 88,
            DialogSize::Xlarge => 116,
        }
    }
}

impl Default for DialogSize {
    fn default() -> Self {
        DialogSize::Medium
    }
}

// ---------------------------------------------------------------------------
// Dialog variant
// ---------------------------------------------------------------------------

/// Which kind of dialog content is being shown.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DialogKind {
    Alert,
    Confirm,
    Select,
    Help,
}

// ---------------------------------------------------------------------------
// Dialog option (for DialogSelect)
// ---------------------------------------------------------------------------

/// A selectable option in a `DialogSelect`.
///
/// Mirrors `DialogSelectOption<T>` from dialog-select.tsx. The generic `T` is
/// replaced by a `String` value for simplicity.
#[derive(Clone)]
pub struct DialogOption {
    pub title: String,
    pub description: Option<String>,
    pub details: Vec<String>,
    pub value: String,
    pub category: Option<String>,
    pub disabled: bool,
}

impl DialogOption {
    pub fn new(title: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            description: None,
            details: vec![],
            value: value.into(),
            category: None,
            disabled: false,
        }
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    pub fn with_category(mut self, cat: impl Into<String>) -> Self {
        self.category = Some(cat.into());
        self
    }

    pub fn with_details(mut self, details: Vec<String>) -> Self {
        self.details = details;
        self
    }
}

// ---------------------------------------------------------------------------
// Dialog result
// ---------------------------------------------------------------------------

/// Result of handling a key event in a dialog.
#[derive(Debug, Clone)]
pub enum DialogResult {
    /// Nothing happened — the dialog consumed the key but no action was triggered.
    None,
    /// User selected an option (value string).
    Select(String),
    /// User confirmed (alert OK / confirm "Yes").
    Confirm,
    /// User cancelled (confirm "No" or pressed Esc).
    Cancel,
    /// Dialog should close.
    Close,
}

// ---------------------------------------------------------------------------
// Confirm focus (for DialogConfirm)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfirmFocus {
    Confirm,
    Cancel,
}

impl ConfirmFocus {
    pub fn toggle(self) -> Self {
        match self {
            ConfirmFocus::Confirm => ConfirmFocus::Cancel,
            ConfirmFocus::Cancel => ConfirmFocus::Confirm,
        }
    }
}

// ---------------------------------------------------------------------------
// Dialog
// ---------------------------------------------------------------------------

/// Dialog state — a stack-aware modal overlay.
///
/// Mirrors the TS `Dialog` component + `DialogProvider` store. The store in TS
/// maintains a stack of elements; here we support a single active dialog which
/// can be pushed/replaced/cleared.
pub struct Dialog {
    pub kind: DialogKind,
    pub title: String,
    pub message: String,
    pub options: Vec<DialogOption>,
    pub selected: usize,
    pub filter: String,
    pub visible: bool,
    pub size: DialogSize,
    pub confirm_focus: ConfirmFocus,
    pub help_text: String,
    pub locked: bool,
    pub created: Instant,
    /// Scroll offset for the select list.
    pub scroll_offset: usize,
}

// ---------------------------------------------------------------------------
// Dialog manager — stack-aware
// ---------------------------------------------------------------------------

/// Manages a stack of dialogs. Mirrors the TS `DialogProvider` store which
/// maintains `stack: { element, onClose }[]` and `size`.
pub struct DialogManager {
    pub stack: Vec<Dialog>,
    pub size: DialogSize,
}

impl DialogManager {
    pub fn new() -> Self {
        Self {
            stack: vec![],
            size: DialogSize::Medium,
        }
    }

    pub fn is_open(&self) -> bool {
        !self.stack.is_empty()
    }

    pub fn current(&self) -> Option<&Dialog> {
        self.stack.last()
    }

    pub fn current_mut(&mut self) -> Option<&mut Dialog> {
        self.stack.last_mut()
    }

    /// Push a new dialog onto the stack.
    pub fn push(&mut self, dialog: Dialog) {
        self.stack.push(dialog);
    }

    /// Replace the entire stack with a single dialog — mirrors `dialog.replace()`.
    pub fn replace(&mut self, dialog: Dialog) {
        self.stack.clear();
        self.size = DialogSize::Medium;
        self.stack.push(dialog);
    }

    /// Clear all dialogs — mirrors `dialog.clear()`.
    pub fn clear(&mut self) {
        self.stack.clear();
        self.size = DialogSize::Medium;
    }

    /// Close the top dialog (pop from stack).
    pub fn close_top(&mut self) {
        self.stack.pop();
    }

    pub fn set_size(&mut self, size: DialogSize) {
        self.size = size;
        if let Some(top) = self.stack.last_mut() {
            top.size = size;
        }
    }

    /// Handle a key event on the top dialog.
    pub fn handle_key(&mut self, key: KeyEvent) -> Option<DialogResult> {
        let dialog = self.stack.last_mut()?;
        Some(dialog.handle_key(key))
    }

    /// Render the top dialog (if any).
    pub fn render(&self, f: &mut Frame, area: Rect, theme: &Theme) {
        if let Some(dialog) = self.stack.last() {
            dialog.render(f, area, theme);
        }
    }
}

impl Default for DialogManager {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Layout helpers
// ---------------------------------------------------------------------------

/// Compute a centered popup area with the given width.
///
/// Mirrors the TS layout:
/// - `paddingTop: dimensions().height / 4` (dialog appears in the upper third)
/// - `width: size.width()`
/// - `maxWidth: dimensions().width - 2`
/// - `alignItems: center`
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

/// Compute a centered rect using percentage constraints (utility).
pub fn centered_rect_percent(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
