//! Border definitions — ported from tui/src/ui/border.ts
//!
//! `EmptyBorder` renders invisible borders (all chars are blank).
//! `SplitBorder` renders only left/right vertical bars using `┃`.

use ratatui::widgets::{Block, Borders};
use ratatui::style::Style;

/// A set of border characters that produce no visible border.
///
/// In the TS source this is a plain object with all fields set to `""`
/// (horizontal is a single space). In ratatui we approximate this by
/// using a `Block` with `Borders::NONE` and a matching style.
pub const EMPTY_BORDERS: Borders = Borders::NONE;

/// The vertical bar character used by `SplitBorder`.
pub const SPLIT_VERTICAL: &str = "┃";

/// Build a block with no visible border.
pub fn empty_border() -> Block<'static> {
    Block::default().borders(EMPTY_BORDERS)
}

/// Build a block that shows only left and right vertical borders using `┃`.
///
/// In the TS source `SplitBorder` is `{ border: ["left","right"], customBorderChars: { ...EmptyBorder, vertical: "┃" } }`.
/// ratatui does not support per-char customization without a custom border set,
/// so we use `Borders::LEFT | Borders::RIGHT` with the default rounded set and
/// override the border type if needed by the caller.
pub fn split_border() -> Block<'static> {
    Block::default()
        .borders(Borders::LEFT | Borders::RIGHT)
        .border_style(Style::default())
}

/// `EmptyBorder` convenience struct — mirrors the TS export shape.
pub struct EmptyBorder;

impl EmptyBorder {
    pub fn block(&self) -> Block<'static> {
        empty_border()
    }
}

/// `SplitBorder` convenience struct — mirrors the TS export shape.
pub struct SplitBorder;

impl SplitBorder {
    pub fn block(&self) -> Block<'static> {
        split_border()
    }

    pub fn borders(&self) -> Borders {
        Borders::LEFT | Borders::RIGHT
    }

    pub fn vertical_char(&self) -> &str {
        SPLIT_VERTICAL
    }
}

/// A custom border set that renders `┃` for verticals and blanks everywhere else.
///
/// This matches the TS `customBorderChars` object:
/// ```ts
/// { ...EmptyBorder, vertical: "┃" }
/// ```
pub fn split_border_set() -> ratatui::symbols::border::Set {
    ratatui::symbols::border::Set {
        top_left: "",
        top_right: "",
        bottom_left: "",
        bottom_right: "",
        vertical_left: "┃",
        vertical_right: "┃",
        horizontal_top: " ",
        horizontal_bottom: " ",
    }
}
