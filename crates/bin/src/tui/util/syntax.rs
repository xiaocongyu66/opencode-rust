//! Syntax highlighting using syntect.
//!
//! Provides syntax-highlighted rendering of code blocks in the TUI.
//! Uses the default dark theme (compatible with most opencode themes).

use std::sync::OnceLock;

use ratatui::style::Color;
use ratatui::text::{Line, Span};
use ratatui::style::Style;
use syntect::highlighting::{Theme as SynTheme, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::easy::HighlightLines;

/// Global syntax set (loaded once).
static SYNTAX_SET: OnceLock<SyntaxSet> = OnceLock::new();

/// Global theme (loaded once).
static SYN_THEME: OnceLock<SynTheme> = OnceLock::new();

fn get_syntax_set() -> &'static SyntaxSet {
    SYNTAX_SET.get_or_init(|| SyntaxSet::load_defaults_newlines())
}

fn get_theme() -> &'static SynTheme {
    SYN_THEME.get_or_init(|| {
        let ts = ThemeSet::load_defaults();
        ts.themes["base16-ocean.dark"].clone()
    })
}

/// Highlight a code string, returning ratatui Lines with colors.
/// `lang` is the language identifier (e.g. "rust", "python", "bash").
/// Falls back to plain text if the language is unknown.
pub fn highlight_code(code: &str, lang: &str) -> Vec<Line<'static>> {
    let ss = get_syntax_set();
    let theme = get_theme();

    // Find the syntax definition for the language.
    let syntax = ss
        .find_syntax_by_token(lang)
        .or_else(|| ss.find_syntax_by_extension(lang))
        .unwrap_or_else(|| ss.find_syntax_plain_text());

    let mut h = HighlightLines::new(syntax, theme);
    let mut lines: Vec<Line<'static>> = Vec::new();

    for line in code.lines() {
        let regions = match h.highlight_line(line, ss) {
            Ok(r) => r,
            Err(_) => {
                lines.push(Line::from(Span::raw(line.to_string())));
                continue;
            }
        };

        let mut spans: Vec<Span<'static>> = Vec::new();
        for (style, text) in regions {
            let fg = convert_color(&style.foreground);
            let rat_style = Style::default().fg(fg);
            spans.push(Span::styled(text.to_string(), rat_style));
        }
        lines.push(Line::from(spans));
    }

    lines
}

/// Convert a syntect color to a ratatui color.
fn convert_color(c: &syntect::highlighting::Color) -> Color {
    if c.a == 0 {
        return Color::Reset;
    }
    Color::Rgb(c.r, c.g, c.b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn highlight_rust_code() {
        let code = "fn main() {\n    println!(\"hello\");\n}";
        let lines = highlight_code(code, "rust");
        assert_eq!(lines.len(), 3);
    }

    #[test]
    fn highlight_unknown_lang_falls_back() {
        let code = "hello world";
        let lines = highlight_code(code, "nonexistent_lang");
        assert_eq!(lines.len(), 1);
    }

    #[test]
    fn highlight_empty_code() {
        let lines = highlight_code("", "rust");
        assert!(lines.is_empty() || lines.len() == 1);
    }
}
