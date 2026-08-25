//! Theme system — ported from opencode TUI theme/assets/*.json
//! 33 themes with full color definitions.
//!
//! Themes are loaded from the upstream JSON files at compile time by
//! `loader.rs`. The `pub fn <name>()` functions below are kept for
//! backwards compatibility but now delegate to the loader.

pub mod loader;
pub mod themes;

use ratatui::style::Color;

/// Theme mode (dark/light)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeMode {
    Dark,
    Light,
}

/// A complete theme definition with all color fields.
#[derive(Debug, Clone)]
pub struct Theme {
    pub primary: Color,
    pub secondary: Color,
    pub accent: Color,
    pub error: Color,
    pub warning: Color,
    pub success: Color,
    pub info: Color,
    pub text: Color,
    pub text_muted: Color,
    pub background: Color,
    pub background_panel: Color,
    pub background_element: Color,
    pub border: Color,
    pub border_active: Color,
    pub border_subtle: Color,
    pub diff_added: Color,
    pub diff_removed: Color,
    pub diff_context: Color,
    pub diff_hunk_header: Color,
    pub diff_highlight_added: Color,
    pub diff_highlight_removed: Color,
    pub diff_added_bg: Color,
    pub diff_removed_bg: Color,
    pub diff_context_bg: Color,
    pub diff_line_number: Color,
    pub diff_added_line_number_bg: Color,
    pub diff_removed_line_number_bg: Color,
    pub markdown_text: Color,
    pub markdown_heading: Color,
    pub markdown_link: Color,
    pub markdown_link_text: Color,
    pub markdown_code: Color,
    pub markdown_block_quote: Color,
    pub markdown_emph: Color,
    pub markdown_strong: Color,
    pub markdown_horizontal_rule: Color,
    pub markdown_list_item: Color,
    pub markdown_list_enumeration: Color,
    pub markdown_image: Color,
    pub markdown_image_text: Color,
    pub markdown_code_block: Color,
    pub syntax_comment: Color,
    pub syntax_keyword: Color,
    pub syntax_function: Color,
    pub syntax_variable: Color,
    pub syntax_string: Color,
    pub syntax_number: Color,
    pub syntax_type: Color,
    pub syntax_operator: Color,
    pub syntax_punctuation: Color,
}


/// Get a theme by name.
pub fn get_theme(name: &str) -> Theme {
    loader::load_theme(name)
}

/// All available theme names.
pub const THEME_NAMES: &[&str] = &[
    "aura",
    "ayu",
    "carbonfox",
    "catppuccin-frappe",
    "catppuccin-macchiato",
    "catppuccin",
    "cobalt2",
    "cursor",
    "dracula",
    "everforest",
    "flexoki",
    "github",
    "gruvbox",
    "kanagawa",
    "lucent-orng",
    "material",
    "matrix",
    "mercury",
    "monokai",
    "nightowl",
    "nord",
    "one-dark",
    "opencode",
    "orng",
    "osaka-jade",
    "palenight",
    "rosepine",
    "solarized",
    "synthwave84",
    "tokyonight",
    "vercel",
    "vesper",
    "zenburn",
];

/// Blend two colors (25% fg over bg).
pub fn tint(bg: Color, fg: Color, alpha: f32) -> Color {
    let (br, bg_, bb) = to_rgb(bg);
    let (fr, fg_, fb) = to_rgb(fg);
    let r = (br as f32 * (1.0 - alpha) + fr as f32 * alpha) as u8;
    let g = (bg_ as f32 * (1.0 - alpha) + fg_ as f32 * alpha) as u8;
    let b = (bb as f32 * (1.0 - alpha) + fb as f32 * alpha) as u8;
    Color::Rgb(r, g, b)
}

fn to_rgb(c: Color) -> (u8, u8, u8) {
    match c {
        Color::Rgb(r, g, b) => (r, g, b),
        _ => (0, 0, 0),
    }
}