//! Built-in fallback theme.
//!
//! All 33 themes are loaded from JSON at compile time by `loader.rs`.
//! This file keeps only the `opencode()` theme as the final fallback when
//! the loader can't find a theme (e.g. the JSON failed to parse). The
//! other 32 theme functions (`aura`, `dracula`, etc.) were removed because
//! the loader supersedes them.

use ratatui::style::Color;

use super::Theme;

/// The default opencode theme (dark). Used as the final fallback when
/// `loader::load_theme` can't resolve a theme name.
pub fn opencode() -> Theme {
    Theme {
        primary: Color::Rgb(250, 178, 131),
        secondary: Color::Rgb(92, 156, 245),
        accent: Color::Rgb(157, 124, 216),
        error: Color::Rgb(224, 108, 117),
        warning: Color::Rgb(245, 167, 66),
        success: Color::Rgb(127, 216, 143),
        info: Color::Rgb(86, 182, 194),
        text: Color::Rgb(238, 238, 238),
        text_muted: Color::Rgb(128, 128, 128),
        background: Color::Rgb(10, 10, 14),
        background_panel: Color::Rgb(28, 28, 38),
        background_element: Color::Rgb(42, 42, 54),
        border: Color::Rgb(88, 88, 108),
        border_active: Color::Rgb(140, 140, 170),
        border_subtle: Color::Rgb(60, 60, 76),
        diff_added: Color::Rgb(79, 214, 190),
        diff_removed: Color::Rgb(197, 59, 83),
        diff_context: Color::Rgb(130, 139, 184),
        diff_hunk_header: Color::Rgb(130, 139, 184),
        diff_highlight_added: Color::Rgb(184, 219, 135),
        diff_highlight_removed: Color::Rgb(226, 106, 117),
        diff_added_bg: Color::Rgb(32, 48, 59),
        diff_removed_bg: Color::Rgb(55, 34, 44),
        diff_context_bg: Color::Rgb(32, 32, 40),
        diff_line_number: Color::Rgb(98, 98, 114),
        diff_added_line_number_bg: Color::Rgb(26, 58, 26),
        diff_removed_line_number_bg: Color::Rgb(58, 26, 26),
        markdown_text: Color::Rgb(238, 238, 238),
        markdown_heading: Color::Rgb(250, 178, 131),
        markdown_link: Color::Rgb(86, 182, 194),
        markdown_link_text: Color::Rgb(92, 156, 245),
        markdown_code: Color::Rgb(127, 216, 143),
        markdown_block_quote: Color::Rgb(128, 128, 128),
        markdown_emph: Color::Rgb(245, 167, 66),
        markdown_strong: Color::Rgb(255, 184, 108),
        markdown_horizontal_rule: Color::Rgb(128, 128, 128),
        markdown_list_item: Color::Rgb(250, 178, 131),
        markdown_list_enumeration: Color::Rgb(86, 182, 194),
        markdown_image: Color::Rgb(86, 182, 194),
        markdown_image_text: Color::Rgb(92, 156, 245),
        markdown_code_block: Color::Rgb(238, 238, 238),
        syntax_comment: Color::Rgb(98, 114, 164),
        syntax_keyword: Color::Rgb(255, 121, 198),
        syntax_function: Color::Rgb(80, 250, 123),
        syntax_variable: Color::Rgb(248, 248, 242),
        syntax_string: Color::Rgb(241, 250, 140),
        syntax_number: Color::Rgb(189, 147, 249),
        syntax_type: Color::Rgb(139, 233, 253),
        syntax_operator: Color::Rgb(255, 121, 198),
        syntax_punctuation: Color::Rgb(220, 220, 204),
    }
}
