//! Prompt input component — the main multi-line text input for user messages.
//! Ported from tui/src/component/prompt/index.tsx + autocomplete.tsx + history.tsx + stash.tsx
//!
//! Features:
//! - Multi-line text editing with cursor movement (left/right/up/down, Home/End, word jumps)
//! - Backspace/Delete, Ctrl+U (delete to line start), Ctrl+K (delete to line end), Ctrl+W (delete word)
//! - Enter submits; Shift+Enter (Alt+Enter / Ctrl+J) inserts a newline
//! - After submit the prompt stays in Insert mode and clears input
//! - Rotating placeholder text ("Ask anything... \"Fix a TODO in the codebase\"" etc.)
//! - Focused border uses `border_active`, unfocused uses `border`
//! - Shell mode (type `!` at cursor offset 0) changes border to `primary` and placeholder
//! - History navigation with Up/Down when cursor is at first/last line
//! - Slash-command autocomplete popup when input starts with `/`
//! - Paste support (Ctrl+V / bracketed paste)
//! - Stash: Ctrl+S saves current draft, Ctrl+L pops stash

pub mod helpers;
pub mod render;

pub use helpers::{Autocomplete, AutocompleteOption};
use helpers::*;

use std::cmp::min;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::Style;
use ratatui::symbols::border::Set as BorderSet;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap};
use ratatui::Frame;

use crate::tui::theme::Theme;

// ---------------------------------------------------------------------------
//  Constants
// ---------------------------------------------------------------------------

/// Rotating placeholder texts shown when the input is empty (normal mode).
/// Mirrors the `placeholders.normal` array from the TS Prompt component.
const PLACEHOLDERS: &[&str] = &[
    "Fix a TODO in the codebase",
    "What is the tech stack of this project?",
    "Fix broken tests",
];

/// Shell-mode placeholder examples.
const SHELL_PLACEHOLDERS: &[&str] = &[
    "ls -la",
    "git status",
    "pwd",
];

/// Min chars before a cleared draft is saved to history.
const DRAFT_RETENTION_MIN_CHARS: usize = 20;

/// Max visible autocomplete items.
const AUTOCOMPLETE_MAX: usize = 10;

/// Max height of the prompt textarea before it starts scrolling.
const PROMPT_MAX_HEIGHT: u16 = 12;

/// Builtin slash commands for autocomplete.
const BUILTIN_COMMANDS: &[(&str, &str)] = &[
    ("sessions", "Browse and resume past sessions"),
    ("new", "Start a new session"),
    ("workspaces", "List and switch workspaces"),
    ("models", "Switch the LLM model"),
    ("agents", "Switch the active agent"),
    ("mcps", "Manage MCP servers"),
    ("variants", "Switch model variant"),
    ("connect", "Connect a provider"),
    ("org", "Switch console organization"),
    ("status", "View session status and cost"),
    ("debug", "Open debug information"),
    ("themes", "Switch color theme"),
    ("help", "Show help and keybindings"),
    ("exit", "Quit rsopencode"),
    ("editor", "Open external editor"),
    ("skills", "Browse and select a skill"),
    ("warp", "Change the workspace for the session"),
    ("move", "Move session to another project dir"),
    ("diff", "View file diff"),
];

// ---------------------------------------------------------------------------
//  PromptInfo — mirrors the TS `PromptInfo` type
// ---------------------------------------------------------------------------

/// A saved prompt snapshot (input + mode).
#[derive(Debug, Clone, Default)]
pub struct PromptInfo {
    pub input: String,
    pub mode: PromptMode,
}

// ---------------------------------------------------------------------------
//  PromptMode
// ---------------------------------------------------------------------------

/// Whether the prompt is in normal or shell mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PromptMode {
    #[default]
    Normal,
    Shell,
}

impl PromptMode {
    pub fn label(&self) -> &'static str {
        match self {
            PromptMode::Normal => "build",
            PromptMode::Shell => "Shell",
        }
    }
}

// ---------------------------------------------------------------------------
//  PromptAction — what `handle_key` returns to the caller
// ---------------------------------------------------------------------------

/// Actions the prompt can emit after processing a key.
#[derive(Debug)]
pub enum PromptAction {
    /// No action — just re-render.
    None,
    /// User pressed Enter (without Shift).  Contains the trimmed text.
    Submit(String),
    /// User pressed Escape (or Backspace at offset 0 in shell mode).
    Cancel,
    /// User typed `exit` / `quit` / `:q`.
    Quit,
}

// ---------------------------------------------------------------------------
//  Cursor position (byte offset based)
// ---------------------------------------------------------------------------

/// A (row, col) pair derived from the byte offset into the multi-line string.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CursorPos {
    pub row: usize,
    pub col: usize,
}

// ---------------------------------------------------------------------------
//  PromptHistory — ring buffer of past prompts
// ---------------------------------------------------------------------------

/// History store for previously submitted or cleared prompts.
#[derive(Debug, Default)]
pub struct PromptHistory {
    entries: Vec<PromptInfo>,
    /// Current navigation index; `None` means "not browsing".
    index: Option<usize>,
    /// The text that was in the buffer when the user first pressed Up.
    draft: Option<String>,
}

impl PromptHistory {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn append(&mut self, info: PromptInfo) {
        if info.input.trim().is_empty() {
            return;
        }
        // Don't append if identical to the last entry.
        if self.entries.last().is_some_and(|e| e.input == info.input) {
            return;
        }
        self.entries.push(info);
        self.index = None;
        self.draft = None;
    }

    /// Move the history cursor by `delta` (−1 for previous, +1 for next).
    /// Returns the prompt to display, or `None` if there is nothing at
    /// that position.
    pub fn navigate(&mut self, delta: i32, current_text: &str) -> Option<PromptInfo> {
        if self.entries.is_empty() {
            return None;
        }

        match self.index {
            None => {
                if delta < 0 {
                    self.draft = Some(current_text.to_string());
                    self.index = Some(self.entries.len() - 1);
                } else {
                    return None;
                }
            }
            Some(idx) => {
                let new_idx = idx as i32 + delta;
                if new_idx < 0 {
                    // Wrap to the latest entry.
                    self.index = Some(self.entries.len() - 1);
                } else if new_idx as usize >= self.entries.len() {
                    // Past the end — restore the draft.
                    self.index = None;
                    return self.draft.take().map(|input| PromptInfo {
                        input,
                        mode: PromptMode::Normal,
                    });
                } else {
                    self.index = Some(new_idx as usize);
                }
            }
        }

        let i = self.index.unwrap();
        self.entries.get(i).cloned()
    }

    /// Reset browsing state (call when the user types or submits).
    pub fn reset(&mut self) {
        self.index = None;
        self.draft = None;
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

// ---------------------------------------------------------------------------
//  PromptStash — save / restore a single draft
// ---------------------------------------------------------------------------

/// A stash entry.
#[derive(Debug, Clone)]
pub struct StashEntry {
    pub input: String,
    pub mode: PromptMode,
}

/// Stack of stashed prompts.
#[derive(Debug, Default)]
pub struct PromptStash {
    entries: Vec<StashEntry>,
}

impl PromptStash {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, entry: StashEntry) {
        self.entries.push(entry);
    }

    pub fn pop(&mut self) -> Option<StashEntry> {
        self.entries.pop()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

// ---------------------------------------------------------------------------
//  Autocomplete — slash-command popup
// ---------------------------------------------------------------------------

/// The Prompt input component.
///
/// Holds the multi-line text buffer, cursor byte offset, focus state,
/// placeholder rotation, shell mode, history, stash, and slash-command
/// autocomplete.
pub struct Prompt {
    /// The raw text buffer (may contain newlines).
    pub input: String,
    /// Byte offset of the cursor into `input`.
    pub cursor: usize,
    /// Whether the prompt is focused (Insert mode).
    pub focused: bool,
    /// Current placeholder index (rotates on each session change / clear).
    placeholder_index: usize,
    /// Shell vs normal mode.
    pub mode: PromptMode,
    /// History store.
    pub history: PromptHistory,
    /// Stash store.
    pub stash: PromptStash,
    /// Autocomplete state.
    pub autocomplete: Autocomplete,
    /// Current agent name (e.g. "Build"). Shown in the meta line.
    pub agent: String,
    /// Current model display name (e.g. "claude-sonnet-4-6").
    pub model: String,
    /// Current provider display name (e.g. "Anthropic").
    pub provider: String,
}

impl Prompt {
    pub fn new() -> Self {
        Self {
            input: String::new(),
            cursor: 0,
            focused: true,
            placeholder_index: random_index(PLACEHOLDERS.len()),
            mode: PromptMode::Normal,
            history: PromptHistory::new(),
            stash: PromptStash::new(),
            autocomplete: Autocomplete::new(),
            agent: "Build".to_string(),
            model: String::new(),
            provider: String::new(),
        }
    }

    // -- Public API ---------------------------------------------------------

    /// Reset placeholder rotation (call on session change).
    pub fn reset_placeholder(&mut self) {
        self.placeholder_index = random_index(PLACEHOLDERS.len());
    }

    /// Clear the input buffer and reset cursor.
    pub fn clear(&mut self) {
        // If the draft is long enough, save it to history before clearing.
        if self.input.trim().len() >= DRAFT_RETENTION_MIN_CHARS {
            self.history.append(PromptInfo {
                input: self.input.clone(),
                mode: self.mode,
            });
        }
        self.input.clear();
        self.cursor = 0;
        self.history.reset();
        self.autocomplete.hide();
    }

    /// Set the input text and move cursor to end.
    pub fn set_text(&mut self, text: impl Into<String>) {
        self.input = text.into();
        self.cursor = self.input.len();
        self.history.reset();
        self.update_autocomplete();
    }

    /// Get the cursor (row, col).
    pub fn cursor_pos(&self) -> CursorPos {
        byte_offset_to_pos(&self.input, self.cursor)
    }

    /// Get the placeholder text for the current mode.
    pub fn placeholder_text(&self) -> String {
        match self.mode {
            PromptMode::Shell => {
                if SHELL_PLACEHOLDERS.is_empty() {
                    return String::new();
                }
                let example = SHELL_PLACEHOLDERS[self.placeholder_index % SHELL_PLACEHOLDERS.len()];
                crate::t!("tui.prompt.run_command", example = example).to_string()
            }
            PromptMode::Normal => {
                if PLACEHOLDERS.is_empty() {
                    return String::new();
                }
                let example = PLACEHOLDERS[self.placeholder_index % PLACEHOLDERS.len()];
                crate::t!("tui.prompt.ask_anything", example = example).to_string()
            }
        }
    }

    /// Enter shell mode.
    pub fn enter_shell_mode(&mut self) {
        self.placeholder_index = random_index(SHELL_PLACEHOLDERS.len());
        self.mode = PromptMode::Shell;
    }

    /// Exit shell mode.
    pub fn exit_shell_mode(&mut self) {
        self.mode = PromptMode::Normal;
    }

    // -- Key handling -------------------------------------------------------

    /// Process a key event and return the resulting action.
    pub fn handle_key(&mut self, key: KeyEvent) -> PromptAction {
        // If autocomplete is visible, intercept navigation keys.
        if self.autocomplete.is_visible() {
            return self.handle_key_with_autocomplete(key);
        }

        match key.code {
            // -- Submit (Enter without Shift/Ctrl) --
            // Shift+Enter / Alt+Enter → insert newline.
            // Ctrl+J is handled separately in handle_ctrl_key.
            // Enter (no modifiers) → submit the entire input (including embedded newlines).
            KeyCode::Enter => {
                if key.modifiers.contains(KeyModifiers::SHIFT)
                    || key.modifiers.contains(KeyModifiers::ALT)
                {
                    self.insert_char('\n');
                    return PromptAction::None;
                }
                return self.submit();
            }

            // -- Escape: exit shell mode or cancel --
            KeyCode::Esc => {
                if self.mode == PromptMode::Shell {
                    self.exit_shell_mode();
                    return PromptAction::None;
                }
                return PromptAction::Cancel;
            }

            // -- Backspace --
            KeyCode::Backspace => {
                // In shell mode at offset 0, exit shell mode instead.
                if self.mode == PromptMode::Shell && self.cursor == 0 && self.input.is_empty() {
                    self.exit_shell_mode();
                    return PromptAction::None;
                }
                self.backspace();
                return PromptAction::None;
            }

            // -- Delete (forward) --
            KeyCode::Delete => {
                self.delete_forward();
                return PromptAction::None;
            }

            // -- Cursor movement --
            KeyCode::Left => {
                self.move_left();
                return PromptAction::None;
            }
            KeyCode::Right => {
                self.move_right();
                return PromptAction::None;
            }
            KeyCode::Up => {
                // History navigation: if cursor is on the first line, go to previous history.
                let pos = self.cursor_pos();
                if pos.row == 0 {
                    return self.history_prev();
                }
                self.move_up();
                return PromptAction::None;
            }
            KeyCode::Down => {
                // History navigation: if cursor is on the last line, go to next history.
                let pos = self.cursor_pos();
                let total = line_count(&self.input);
                if pos.row >= total.saturating_sub(1) {
                    return self.history_next();
                }
                self.move_down();
                return PromptAction::None;
            }
            KeyCode::Home => {
                self.move_line_start();
                return PromptAction::None;
            }
            KeyCode::End => {
                self.move_line_end();
                return PromptAction::None;
            }

            // -- Ctrl+U: delete to line start --
            // -- Ctrl+K: delete to line end --
            // -- Ctrl+W: delete previous word --
            // -- Ctrl+V: paste --
            // -- Ctrl+S: stash --
            // -- Ctrl+L: pop stash --
            KeyCode::Char(c) => {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    return self.handle_ctrl_key(c);
                }
                // Regular character
                self.insert_char(c);
                self.update_autocomplete();
                PromptAction::None
            }

            _ => PromptAction::None,
        }
    }

    /// Handle keys when autocomplete popup is visible.
    fn handle_key_with_autocomplete(&mut self, key: KeyEvent) -> PromptAction {
        match key.code {
            KeyCode::Up => {
                self.autocomplete.move_selection(-1);
                PromptAction::None
            }
            KeyCode::Down => {
                self.autocomplete.move_selection(1);
                PromptAction::None
            }
            KeyCode::Enter | KeyCode::Tab => {
                self.select_autocomplete();
                PromptAction::None
            }
            KeyCode::Esc => {
                self.autocomplete.hide();
                PromptAction::None
            }
            KeyCode::Backspace => {
                self.backspace();
                self.update_autocomplete();
                if self.should_hide_autocomplete() {
                    self.autocomplete.hide();
                }
                PromptAction::None
            }
            KeyCode::Char(c) => {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    return self.handle_ctrl_key(c);
                }
                self.insert_char(c);
                self.update_autocomplete();
                if self.should_hide_autocomplete() {
                    self.autocomplete.hide();
                }
                PromptAction::None
            }
            _ => PromptAction::None,
        }
    }

    /// Handle Ctrl+key shortcuts.
    fn handle_ctrl_key(&mut self, c: char) -> PromptAction {
        match c {
            // Ctrl+C → cancel
            'c' => PromptAction::Cancel,
            // Ctrl+U → delete to line start
            'u' => {
                let pos = self.cursor_pos();
                let start = line_start(&self.input, pos.row);
                if start < self.cursor {
                    self.input.drain(start..self.cursor);
                    self.cursor = start;
                    self.history.reset();
                    self.update_autocomplete();
                }
                PromptAction::None
            }
            // Ctrl+K → delete to line end
            'k' => {
                let pos = self.cursor_pos();
                let end = line_end(&self.input, pos.row);
                if end > self.cursor {
                    self.input.drain(self.cursor..end);
                    self.history.reset();
                    self.update_autocomplete();
                }
                PromptAction::None
            }
            // Ctrl+W → delete previous word
            'w' => {
                self.delete_word_back();
                PromptAction::None
            }
            // Ctrl+A → move to line start (like readline)
            'a' => {
                self.move_line_start();
                PromptAction::None
            }
            // Ctrl+E → move to line end (like readline)
            'e' => {
                self.move_line_end();
                PromptAction::None
            }
            // Ctrl+V → paste from clipboard (placeholder)
            'v' => {
                // In a real implementation this would read the system clipboard.
                // For now, do nothing — bracketed paste is handled at the app level.
                PromptAction::None
            }
            // Ctrl+S → stash current prompt
            's' => {
                self.stash_prompt();
                PromptAction::None
            }
            // Ctrl+L → pop stash
            'l' => {
                self.pop_stash();
                PromptAction::None
            }
            // Ctrl+J → insert newline (alternative to Shift+Enter)
            'j' => {
                self.insert_char('\n');
                PromptAction::None
            }
            _ => PromptAction::None,
        }
    }

    // -- Text editing primitives -------------------------------------------

    /// Insert a character at the cursor position.
    fn insert_char(&mut self, c: char) {
        // Check for shell-mode trigger: `!` at offset 0 in normal mode.
        if c == '!'
            && self.mode == PromptMode::Normal
            && self.cursor == 0
            && self.input.is_empty()
        {
            self.enter_shell_mode();
            return;
        }

        self.input.insert(self.cursor, c);
        self.cursor += c.len_utf8();
        self.history.reset();
    }

    /// Delete the character before the cursor (Backspace).
    fn backspace(&mut self) {
        if self.cursor == 0 {
            return;
        }
        // Find the start of the previous char.
        let prev = self.input[..self.cursor]
            .char_indices()
            .last()
            .map(|(i, _)| i)
            .unwrap_or(0);
        self.input.replace_range(prev..self.cursor, "");
        self.cursor = prev;
        self.history.reset();
    }

    /// Delete the character at the cursor (Delete key).
    fn delete_forward(&mut self) {
        if self.cursor >= self.input.len() {
            return;
        }
        let next = self.input[self.cursor..]
            .char_indices()
            .nth(1)
            .map(|(i, _)| self.cursor + i)
            .unwrap_or(self.input.len());
        self.input.replace_range(self.cursor..next, "");
        self.history.reset();
    }

    /// Delete the word before the cursor (Ctrl+W).
    fn delete_word_back(&mut self) {
        if self.cursor == 0 {
            return;
        }
        let text = &self.input[..self.cursor];
        let chars: Vec<(usize, char)> = text.char_indices().collect();
        let mut i = chars.len();
        // Skip whitespace backwards.
        while i > 0 && chars[i - 1].1.is_whitespace() {
            i -= 1;
        }
        // Skip non-whitespace backwards.
        while i > 0 && !chars[i - 1].1.is_whitespace() {
            i -= 1;
        }
        let start = if i > 0 { chars[i - 1].0 + chars[i - 1].1.len_utf8() } else { 0 };
        let end = self.cursor;
        self.input.replace_range(start..end, "");
        self.cursor = start;
        self.history.reset();
        self.update_autocomplete();
    }

    // -- Cursor movement primitives ----------------------------------------

    /// Move cursor left one char.
    fn move_left(&mut self) {
        if self.cursor == 0 {
            return;
        }
        let prev = self.input[..self.cursor]
            .char_indices()
            .last()
            .map(|(i, _)| i)
            .unwrap_or(0);
        self.cursor = prev;
    }

    /// Move cursor right one char.
    fn move_right(&mut self) {
        if self.cursor >= self.input.len() {
            return;
        }
        let next = self.input[self.cursor..]
            .char_indices()
            .nth(1)
            .map(|(i, _)| self.cursor + i)
            .unwrap_or(self.input.len());
        self.cursor = next;
    }

    /// Move cursor up one line (keeping column if possible).
    fn move_up(&mut self) {
        let pos = self.cursor_pos();
        if pos.row == 0 {
            self.cursor = 0;
            return;
        }
        let target_row = pos.row - 1;
        let target_col = display_width(line_text(&self.input, target_row)).min(pos.col);
        self.cursor = pos_to_byte_offset(&self.input, CursorPos { row: target_row, col: target_col });
    }

    /// Move cursor down one line (keeping column if possible).
    fn move_down(&mut self) {
        let pos = self.cursor_pos();
        let total = line_count(&self.input);
        if pos.row >= total.saturating_sub(1) {
            self.cursor = self.input.len();
            return;
        }
        let target_row = pos.row + 1;
        let target_col = display_width(line_text(&self.input, target_row)).min(pos.col);
        self.cursor = pos_to_byte_offset(&self.input, CursorPos { row: target_row, col: target_col });
    }

    /// Move cursor to the start of the current line.
    fn move_line_start(&mut self) {
        let pos = self.cursor_pos();
        self.cursor = line_start(&self.input, pos.row);
    }

    /// Move cursor to the end of the current line.
    fn move_line_end(&mut self) {
        let pos = self.cursor_pos();
        self.cursor = line_end(&self.input, pos.row);
    }

    // -- History navigation -------------------------------------------------

    /// Navigate to the previous history entry.
    fn history_prev(&mut self) -> PromptAction {
        if let Some(info) = self.history.navigate(-1, &self.input) {
            self.input = info.input.clone();
            self.cursor = 0;
            self.mode = info.mode;
            self.update_autocomplete();
        }
        PromptAction::None
    }

    /// Navigate to the next history entry.
    fn history_next(&mut self) -> PromptAction {
        if let Some(info) = self.history.navigate(1, &self.input) {
            self.input = info.input.clone();
            self.cursor = self.input.len();
            self.mode = info.mode;
            self.update_autocomplete();
        }
        PromptAction::None
    }

    // -- Stash ---------------------------------------------------------------

    /// Stash the current prompt text.
    fn stash_prompt(&mut self) {
        if self.input.is_empty() {
            return;
        }
        self.stash.push(StashEntry {
            input: std::mem::take(&mut self.input),
            mode: self.mode,
        });
        self.cursor = 0;
        self.autocomplete.hide();
    }

    /// Pop the most recently stashed prompt.
    fn pop_stash(&mut self) {
        if let Some(entry) = self.stash.pop() {
            self.input = entry.input;
            self.cursor = self.input.len();
            self.mode = entry.mode;
            self.update_autocomplete();
        }
    }

    // -- Autocomplete -------------------------------------------------------

    /// Check if the current input should trigger autocomplete and update state.
    fn update_autocomplete(&mut self) {
        if self.autocomplete.is_visible() {
            // Check if we should hide: cursor before trigger, or space after trigger.
            if self.cursor <= self.autocomplete.trigger_index {
                self.autocomplete.hide();
                return;
            }
            let between = &self.input[self.autocomplete.trigger_index..self.cursor.min(self.input.len())];
            if between.contains(' ') || between.contains('\n') {
                self.autocomplete.hide();
                return;
            }
            // Update filter with the text after the trigger char.
            let filter = &self.input[self.autocomplete.trigger_index..self.cursor.min(self.input.len())];
            self.autocomplete.set_filter(filter);
            return;
        }

        // Check if we should show: input starts with `/` and no space before cursor.
        if self.input.starts_with('/') && self.cursor > 0 {
            let before_cursor = &self.input[..self.cursor];
            if !before_cursor.contains(' ') && !before_cursor.contains('\n') {
                self.autocomplete.show('/', 0);
                let filter = &self.input[0..self.cursor];
                self.autocomplete.set_filter(filter);
            }
        }
    }

    /// Check if autocomplete should be hidden based on current input.
    fn should_hide_autocomplete(&self) -> bool {
        if !self.input.starts_with('/') {
            return true;
        }
        if self.cursor == 0 {
            return true;
        }
        let before_cursor = &self.input[..self.cursor];
        // Hide if there's a space after the slash and before the cursor (command is complete).
        if before_cursor.len() > 1 && before_cursor[1..].contains(' ') {
            return true;
        }
        false
    }

    /// Select the currently highlighted autocomplete option.
    fn select_autocomplete(&mut self) {
        if let Some(opt) = self.autocomplete.selected_option() {
            let display = &opt.display;
            // Replace the current `/query` text with the selected command + space.
            let new_text = format!("{} ", display);
            self.input = new_text.clone();
            self.cursor = self.input.len();
        }
        self.autocomplete.hide();
    }

    // -- Submit --------------------------------------------------------------

    /// Attempt to submit the current input.  Returns `Submit` if the input
    /// was non-empty, or `None` / `Quit` otherwise.
    fn submit(&mut self) -> PromptAction {
        let text = self.input.trim().to_string();

        // Handle exit commands.
        if text == "exit" || text == "quit" || text == ":q" {
            return PromptAction::Quit;
        }

        if text.is_empty() {
            return PromptAction::None;
        }

        // Save to history.
        self.history.append(PromptInfo {
            input: self.input.clone(),
            mode: self.mode,
        });

        // Exit shell mode after submit.
        if self.mode == PromptMode::Shell {
            self.exit_shell_mode();
        }

        // Clear input, keep Insert mode.
        self.input.clear();
        self.cursor = 0;
        self.history.reset();
        self.autocomplete.hide();

        PromptAction::Submit(text)
    }


    /// Render the autocomplete popup (call after `render`).
    pub fn render_autocomplete(&self, f: &mut Frame, area: Rect, theme: &Theme) {
        self.autocomplete.render(f, area, theme);
    }
}

impl Default for Prompt {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
//  Helpers
// ---------------------------------------------------------------------------

/// Generate a random index in `[0, count)`.
fn random_index(count: usize) -> usize {
    if count == 0 {
        return 0;
    }
    // Use a simple thread-local RNG to avoid pulling in rand just for this.
    use std::cell::Cell;
    thread_local! {
        static SEED: Cell<u64> = Cell::new(0x1234_5678_9abc_def0);
    }
    SEED.with(|s| {
        let mut x = s.get();
        // xorshift64
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        s.set(x);
        (x % count as u64) as usize
    })
}
