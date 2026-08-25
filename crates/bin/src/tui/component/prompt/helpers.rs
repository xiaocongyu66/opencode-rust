use super::*;
#[derive(Debug, Clone)]
pub struct AutocompleteOption {
    pub display: String,
    pub description: Option<String>,
}

/// Autocomplete state for the slash-command popup.
#[derive(Debug, Default)]
pub struct Autocomplete {
    /// `false` = hidden, `Some("/")` = slash commands visible.
    pub visible: Option<char>,
    pub options: Vec<AutocompleteOption>,
    pub selected: usize,
    /// Byte offset where the trigger char (`/`) was typed.
    pub trigger_index: usize,
    /// Current filter text after the trigger.
    pub filter: String,
}

impl Autocomplete {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_visible(&self) -> bool {
        self.visible.is_some()
    }

    /// Show the popup for the given trigger char.
    pub fn show(&mut self, trigger: char, index: usize) {
        self.visible = Some(trigger);
        self.trigger_index = index;
        self.selected = 0;
        self.filter.clear();
        self.rebuild_options();
    }

    /// Hide the popup.
    pub fn hide(&mut self) {
        self.visible = None;
        self.filter.clear();
        self.selected = 0;
    }

    /// Rebuild the option list based on the current filter.
    pub fn rebuild_options(&mut self) {
        let all: Vec<AutocompleteOption> = BUILTIN_COMMANDS
            .iter()
            .map(|(name, desc)| AutocompleteOption {
                display: format!("/{}", name),
                description: Some(desc.to_string()),
            })
            .collect();

        if self.filter.is_empty() {
            self.options = all;
        } else {
            let f = self.filter.to_lowercase();
            // Strip the leading "/" from filter for matching.
            let f = f.trim_start_matches('/');
            self.options = all
                .into_iter()
                .filter(|o| {
                    o.display.to_lowercase().contains(&f)
                        || o
                            .description
                            .as_ref()
                            .is_some_and(|d| d.to_lowercase().contains(&f))
                })
                .collect();
        }
    }

    /// Update filter text and rebuild.
    pub fn set_filter(&mut self, filter: &str) {
        self.filter = filter.to_string();
        self.selected = 0;
        self.rebuild_options();
    }

    /// Move selection by `delta`.
    pub fn move_selection(&mut self, delta: i32) {
        if self.options.is_empty() {
            return;
        }
        let len = self.options.len() as i32;
        let mut next = self.selected as i32 + delta;
        if next < 0 {
            next = len - 1;
        }
        if next >= len {
            next = 0;
        }
        self.selected = next as usize;
    }

    /// Get the currently selected option.
    pub fn selected_option(&self) -> Option<&AutocompleteOption> {
        self.options.get(self.selected)
    }

    /// Render the autocomplete popup above the prompt.
    pub fn render(&self, f: &mut Frame, prompt_area: Rect, theme: &Theme) {
        if self.visible.is_none() {
            return;
        }

        let count = min(self.options.len(), AUTOCOMPLETE_MAX);
        if count == 0 {
            // Show "No matching items"
            let popup_area = Rect {
                x: prompt_area.x,
                y: prompt_area.y.saturating_sub(2),
                width: prompt_area.width,
                height: 1,
            };
            f.render_widget(Clear, popup_area);
            f.render_widget(
                Paragraph::new(format!("  {}", crate::t!("tui.prompt.no_matching")))
                    .style(Style::default().fg(theme.text_muted).bg(theme.background_element)),
                popup_area,
            );
            return;
        }

        // Sliding window: show AUTOCOMPLETE_MAX items centered on `selected`
        // so the user can scroll through all options.
        let window_start = if self.options.len() <= AUTOCOMPLETE_MAX {
            0
        } else if self.selected < AUTOCOMPLETE_MAX / 2 {
            0
        } else if self.selected >= self.options.len() - AUTOCOMPLETE_MAX / 2 {
            self.options.len() - AUTOCOMPLETE_MAX
        } else {
            self.selected - AUTOCOMPLETE_MAX / 2
        };

        let height = count as u16;
        let popup_area = Rect {
            x: prompt_area.x,
            y: prompt_area.y.saturating_sub(height),
            width: prompt_area.width,
            height,
        };

        f.render_widget(Clear, popup_area);

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border))
            .style(Style::default().bg(theme.background_element));

        let inner = {
            let mut a = popup_area;
            a.x += 1;
            a.y += 1;
            a.width = a.width.saturating_sub(2);
            a.height = a.height.saturating_sub(2);
            a
        };

        f.render_widget(block, popup_area);

        let items: Vec<ListItem> = self
            .options
            .iter()
            .skip(window_start)
            .take(AUTOCOMPLETE_MAX)
            .enumerate()
            .map(|(i, opt)| {
                let abs_idx = window_start + i;
                let is_selected = abs_idx == self.selected;
                let fg = if is_selected {
                    theme.primary
                } else {
                    theme.text
                };
                let desc_fg = if is_selected {
                    theme.primary
                } else {
                    theme.text_muted
                };
                let mut spans = vec![Span::styled(
                    opt.display.clone(),
                    Style::default().fg(fg),
                )];
                if let Some(ref desc) = opt.description {
                    spans.push(Span::raw("  "));
                    spans.push(Span::styled(desc.clone(), Style::default().fg(desc_fg)));
                }
                ListItem::new(Line::from(spans))
            })
            .collect();

        f.render_widget(List::new(items), inner);
    }
}

// ---------------------------------------------------------------------------
//  Cursor helpers — work on byte offsets into a UTF-8 string
// ---------------------------------------------------------------------------

/// Convert a byte offset into a (row, col) position.
pub(super) fn byte_offset_to_pos(text: &str, offset: usize) -> CursorPos {
    let offset = offset.min(text.len());
    let mut row = 0usize;
    let mut col = 0usize;
    for (i, ch) in text.char_indices() {
        if i >= offset {
            break;
        }
        if ch == '\n' {
            row += 1;
            col = 0;
        } else {
            col += 1;
        }
    }
    CursorPos { row, col }
}

/// Convert a (row, col) position back to a byte offset.
pub(super) fn pos_to_byte_offset(text: &str, pos: CursorPos) -> usize {
    let mut byte = 0usize;
    let mut row = 0usize;
    let mut col = 0usize;
    for (i, ch) in text.char_indices() {
        if row == pos.row && col >= pos.col {
            return i;
        }
        if ch == '\n' {
            if row == pos.row {
                return i;
            }
            row += 1;
            col = 0;
        } else {
            col += 1;
        }
        byte = i + ch.len_utf8();
    }
    if row == pos.row && col >= pos.col {
        return text.len();
    }
    byte
}

/// Count the number of lines in `text`.
pub(super) fn line_count(text: &str) -> usize {
    if text.is_empty() {
        return 1;
    }
    text.lines().count() + if text.ends_with('\n') { 1 } else { 0 }
}

/// Get the start byte offset of line `row`.
pub(super) fn line_start(text: &str, row: usize) -> usize {
    if row == 0 {
        return 0;
    }
    let mut current_row = 0usize;
    for (i, ch) in text.char_indices() {
        if ch == '\n' {
            current_row += 1;
            if current_row == row {
                return i + 1;
            }
        }
    }
    text.len()
}

/// Get the byte offset of the end of line `row` (exclusive of the `\n`).
pub(super) fn line_end(text: &str, row: usize) -> usize {
    let start = line_start(text, row);
    let rest = &text[start..];
    match rest.find('\n') {
        Some(i) => start + i,
        None => text.len(),
    }
}

/// Get the text of line `row`.
pub(super) fn line_text<'a>(text: &'a str, row: usize) -> &'a str {
    let start = line_start(text, row);
    let end = line_end(text, row);
    &text[start..end]
}

/// Count display-width of a string (char count — simplified, not full-width aware).
pub(super) fn display_width(s: &str) -> usize {
    s.chars().count()
}

// ---------------------------------------------------------------------------
//  Prompt — the main component
// ---------------------------------------------------------------------------

