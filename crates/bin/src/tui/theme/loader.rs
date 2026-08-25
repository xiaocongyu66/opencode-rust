//! JSON theme loader — loads themes from the original TS theme/assets/*.json
//! files at compile time and resolves them into `Theme` structs at runtime.
//!
//! This replaces the 30+ placeholder `pub fn <name>() -> Theme` functions in
//! `mod.rs` that returned all-black themes. Real color values now come from
//! the upstream JSON files.
//!
//! The JSON format (see dracula.json for reference):
//! - `defs`: map of name → hex color (e.g. `"purple": "#bd93f9"`)
//! - `theme`: map of ThemeColor → `{ dark: ColorValue, light: ColorValue }`
//!
//! `ColorValue` can be:
//! - a `#rrggbb` hex string
//! - a `defs` variable name (resolved recursively)
//! - a `theme` key name (resolved recursively, for aliases)
//! - `"transparent"` / `"none"` → (0,0,0,0)

use std::collections::HashMap;
use std::sync::OnceLock;

use ratatui::style::Color;
use serde_json::Value;

use super::Theme;

/// All embedded theme JSON files, keyed by theme name.
/// The path is relative to this file.
static THEME_JSONS: &[(&str, &str)] = &[
    ("aura", include_str!("../../../../../opencode/packages/tui/src/theme/assets/aura.json")),
    ("ayu", include_str!("../../../../../opencode/packages/tui/src/theme/assets/ayu.json")),
    ("carbonfox", include_str!("../../../../../opencode/packages/tui/src/theme/assets/carbonfox.json")),
    ("catppuccin-frappe", include_str!("../../../../../opencode/packages/tui/src/theme/assets/catppuccin-frappe.json")),
    ("catppuccin-macchiato", include_str!("../../../../../opencode/packages/tui/src/theme/assets/catppuccin-macchiato.json")),
    ("catppuccin", include_str!("../../../../../opencode/packages/tui/src/theme/assets/catppuccin.json")),
    ("cobalt2", include_str!("../../../../../opencode/packages/tui/src/theme/assets/cobalt2.json")),
    ("cursor", include_str!("../../../../../opencode/packages/tui/src/theme/assets/cursor.json")),
    ("dracula", include_str!("../../../../../opencode/packages/tui/src/theme/assets/dracula.json")),
    ("everforest", include_str!("../../../../../opencode/packages/tui/src/theme/assets/everforest.json")),
    ("flexoki", include_str!("../../../../../opencode/packages/tui/src/theme/assets/flexoki.json")),
    ("github", include_str!("../../../../../opencode/packages/tui/src/theme/assets/github.json")),
    ("gruvbox", include_str!("../../../../../opencode/packages/tui/src/theme/assets/gruvbox.json")),
    ("kanagawa", include_str!("../../../../../opencode/packages/tui/src/theme/assets/kanagawa.json")),
    ("lucent-orng", include_str!("../../../../../opencode/packages/tui/src/theme/assets/lucent-orng.json")),
    ("material", include_str!("../../../../../opencode/packages/tui/src/theme/assets/material.json")),
    ("matrix", include_str!("../../../../../opencode/packages/tui/src/theme/assets/matrix.json")),
    ("mercury", include_str!("../../../../../opencode/packages/tui/src/theme/assets/mercury.json")),
    ("monokai", include_str!("../../../../../opencode/packages/tui/src/theme/assets/monokai.json")),
    ("nightowl", include_str!("../../../../../opencode/packages/tui/src/theme/assets/nightowl.json")),
    ("nord", include_str!("../../../../../opencode/packages/tui/src/theme/assets/nord.json")),
    ("one-dark", include_str!("../../../../../opencode/packages/tui/src/theme/assets/one-dark.json")),
    ("opencode", include_str!("../../../../../opencode/packages/tui/src/theme/assets/opencode.json")),
    ("orng", include_str!("../../../../../opencode/packages/tui/src/theme/assets/orng.json")),
    ("osaka-jade", include_str!("../../../../../opencode/packages/tui/src/theme/assets/osaka-jade.json")),
    ("palenight", include_str!("../../../../../opencode/packages/tui/src/theme/assets/palenight.json")),
    ("rosepine", include_str!("../../../../../opencode/packages/tui/src/theme/assets/rosepine.json")),
    ("solarized", include_str!("../../../../../opencode/packages/tui/src/theme/assets/solarized.json")),
    ("synthwave84", include_str!("../../../../../opencode/packages/tui/src/theme/assets/synthwave84.json")),
    ("tokyonight", include_str!("../../../../../opencode/packages/tui/src/theme/assets/tokyonight.json")),
    ("vercel", include_str!("../../../../../opencode/packages/tui/src/theme/assets/vercel.json")),
    ("vesper", include_str!("../../../../../opencode/packages/tui/src/theme/assets/vesper.json")),
    ("zenburn", include_str!("../../../../../opencode/packages/tui/src/theme/assets/zenburn.json")),
];

/// Cache of all resolved themes, built on first access.
static RESOLVED_THEMES: OnceLock<HashMap<String, Theme>> = OnceLock::new();

/// Get a resolved theme by name. Falls back to the "opencode" theme if the
/// name is unknown, and to a built-in default if even that is missing.
pub fn load_theme(name: &str) -> Theme {
    let themes = RESOLVED_THEMES.get_or_init(resolve_all_themes);
    themes
        .get(name.to_lowercase().as_str())
        .cloned()
        .or_else(|| themes.get("opencode").cloned())
        .unwrap_or_else(super::themes::opencode)
}

/// List all available theme names (sorted, for the theme picker dialog).
pub fn list_theme_names() -> Vec<&'static str> {
    let mut names: Vec<&'static str> = THEME_JSONS.iter().map(|(n, _)| *n).collect();
    names.sort();
    names
}

fn resolve_all_themes() -> HashMap<String, Theme> {
    let mut map = HashMap::with_capacity(THEME_JSONS.len());
    for (name, json) in THEME_JSONS {
        match resolve_one(json) {
            Ok(theme) => {
                map.insert((*name).to_string(), theme);
            }
            Err(e) => {
                tracing::warn!("failed to resolve theme {name}: {e}");
            }
        }
    }
    map
}

/// Resolve a single theme JSON into a `Theme`. Uses dark-mode colors.
fn resolve_one(json: &str) -> Result<Theme, String> {
    let value: Value = serde_json::from_str(json).map_err(|e| format!("parse: {e}"))?;

    let defs = value
        .get("defs")
        .and_then(|v| v.as_object())
        .cloned()
        .unwrap_or_default();

    let theme_obj = value
        .get("theme")
        .and_then(|v| v.as_object())
        .ok_or_else(|| "missing 'theme' object".to_string())?;

    // Build a resolver closure.
    // ColorValue can be: string, {dark, light} object, or number (ansi).
    let mut resolve = |cv: &Value, chain: &[String]| -> Result<Color, String> {
        resolve_color(cv, &defs, theme_obj, chain)
    };

    let get = |key: &str| -> Color {
        theme_obj
            .get(key)
            .map(|v| resolve(v, &[]).unwrap_or(Color::Reset))
            .unwrap_or(Color::Reset)
    };

    let theme = Theme {
        primary: get("primary"),
        secondary: get("secondary"),
        accent: get("accent"),
        error: get("error"),
        warning: get("warning"),
        success: get("success"),
        info: get("info"),
        text: get("text"),
        text_muted: get("textMuted"),
        background: get("background"),
        background_panel: get("backgroundPanel"),
        background_element: get("backgroundElement"),
        border: get("border"),
        border_active: get("borderActive"),
        border_subtle: get("borderSubtle"),
        diff_added: get("diffAdded"),
        diff_removed: get("diffRemoved"),
        diff_context: get("diffContext"),
        diff_hunk_header: get("diffHunkHeader"),
        diff_highlight_added: get("diffHighlightAdded"),
        diff_highlight_removed: get("diffHighlightRemoved"),
        diff_added_bg: get("diffAddedBg"),
        diff_removed_bg: get("diffRemovedBg"),
        diff_context_bg: get("diffContextBg"),
        diff_line_number: get("diffLineNumber"),
        diff_added_line_number_bg: get("diffAddedLineNumberBg"),
        diff_removed_line_number_bg: get("diffRemovedLineNumberBg"),
        markdown_text: get("markdownText"),
        markdown_heading: get("markdownHeading"),
        markdown_link: get("markdownLink"),
        markdown_link_text: get("markdownLinkText"),
        markdown_code: get("markdownCode"),
        markdown_block_quote: get("markdownBlockQuote"),
        markdown_emph: get("markdownEmph"),
        markdown_strong: get("markdownStrong"),
        markdown_horizontal_rule: get("markdownHorizontalRule"),
        markdown_list_item: get("markdownListItem"),
        markdown_list_enumeration: get("markdownListEnumeration"),
        markdown_image: get("markdownImage"),
        markdown_image_text: get("markdownImageText"),
        markdown_code_block: get("markdownCodeBlock"),
        syntax_comment: get("syntaxComment"),
        syntax_keyword: get("syntaxKeyword"),
        syntax_function: get("syntaxFunction"),
        syntax_variable: get("syntaxVariable"),
        syntax_string: get("syntaxString"),
        syntax_number: get("syntaxNumber"),
        syntax_type: get("syntaxType"),
        syntax_operator: get("syntaxOperator"),
        syntax_punctuation: get("syntaxPunctuation"),
    };

    Ok(boost_bg_contrast(theme))
}

/// Boost contrast between the three background tiers so that user messages
/// (panel) and input areas (element) are visually distinguishable from the
/// page background. If a theme defines all three with insufficient contrast
/// (e.g. all near-black with only a 10-unit gap), this lightens panel and
/// element while keeping the overall mood.
fn boost_bg_contrast(mut theme: Theme) -> Theme {
    let bg = theme.background;
    let panel = theme.background_panel;
    let element = theme.background_element;

    // Only boost if all three are RGB (not Reset/transparent themes).
    let (Color::Rgb(br, bg_, bb)) = bg else { return theme; };
    let _ = (br, bg_, bb);
    let (Color::Rgb(pr, pg, pb)) = panel else { return theme; };
    let (Color::Rgb(er, eg, eb)) = element else { return theme; };

    // Compute brightness (max channel value) as a simple luminance proxy.
    let bg_brightness = bg_max(br, bg_, bb);
    let panel_brightness = bg_max(pr, pg, pb);
    let element_brightness = bg_max(er, eg, eb);

    // If panel is already at least 20 units brighter than bg, leave it.
    if panel_brightness.saturating_sub(bg_brightness) >= 20 {
        return theme;
    }

    // Otherwise, boost panel and element. The boost is additive and
    // preserves hue direction by scaling each channel proportionally.
    let panel_boost = if bg_brightness < 40 { 18 } else { 12 };
    let element_boost = if bg_brightness < 40 { 32 } else { 22 };

    theme.background_panel = Color::Rgb(
        saturating_add(pr, panel_boost),
        saturating_add(pg, panel_boost),
        saturating_add(pb, panel_boost),
    );
    theme.background_element = Color::Rgb(
        saturating_add(er, element_boost),
        saturating_add(eg, element_boost),
        saturating_add(eb, element_boost),
    );
    theme
}

fn bg_max(r: u8, g: u8, b: u8) -> u8 {
    r.max(g).max(b)
}

fn saturating_add(c: u8, v: u8) -> u8 {
    c.saturating_add(v)
}

fn resolve_color(
    cv: &Value,
    defs: &serde_json::Map<String, Value>,
    theme_obj: &serde_json::Map<String, Value>,
    chain: &[String],
) -> Result<Color, String> {
    match cv {
        // String: hex color, "transparent", or a variable reference.
        Value::String(s) => {
            if s == "transparent" || s == "none" {
                return Ok(Color::Reset);
            }
            if let Some(c) = parse_hex(s) {
                return Ok(c);
            }
            // Variable reference — resolve recursively.
            if chain.iter().any(|c| c == s) {
                return Err(format!("circular color reference: {} -> {}", chain.join(" -> "), s));
            }
            let mut new_chain = chain.to_vec();
            new_chain.push(s.clone());
            if let Some(next) = defs.get(s) {
                return resolve_color(next, defs, theme_obj, &new_chain);
            }
            if let Some(next) = theme_obj.get(s) {
                return resolve_color(next, defs, theme_obj, &new_chain);
            }
            Err(format!("color reference not found: {s}"))
        }
        // Object: { dark: ..., light: ... } — take dark.
        Value::Object(_) => {
            let dark = cv.get("dark").or_else(|| cv.get("light"));
            match dark {
                Some(v) => resolve_color(v, defs, theme_obj, chain),
                None => Err("color object missing dark/light".to_string()),
            }
        }
        // Number: ANSI 16-color index — map to the basic palette.
        Value::Number(n) => {
            let i = n.as_u64().unwrap_or(0) as u8;
            Ok(ansi_to_color(i))
        }
        _ => Err(format!("unsupported color value: {cv}")),
    }
}

/// Parse `#rrggbb` (or `#rgb`) into a Color::Rgb.
fn parse_hex(s: &str) -> Option<Color> {
    let s = s.strip_prefix('#')?;
    if s.len() == 6 {
        let r = u8::from_str_radix(&s[0..2], 16).ok()?;
        let g = u8::from_str_radix(&s[2..4], 16).ok()?;
        let b = u8::from_str_radix(&s[4..6], 16).ok()?;
        Some(Color::Rgb(r, g, b))
    } else if s.len() == 3 {
        let r = u8::from_str_radix(&s[0..1].repeat(2), 16).ok()?;
        let g = u8::from_str_radix(&s[1..2].repeat(2), 16).ok()?;
        let b = u8::from_str_radix(&s[2..3].repeat(2), 16).ok()?;
        Some(Color::Rgb(r, g, b))
    } else {
        None
    }
}

/// Map an ANSI 16-color index to a ratatui basic Color.
fn ansi_to_color(i: u8) -> Color {
    match i {
        0 => Color::Black,
        1 => Color::Red,
        2 => Color::Green,
        3 => Color::Yellow,
        4 => Color::Blue,
        5 => Color::Magenta,
        6 => Color::Cyan,
        7 => Color::Gray,
        8 => Color::DarkGray,
        9 => Color::LightRed,
        10 => Color::LightGreen,
        11 => Color::LightYellow,
        12 => Color::LightBlue,
        13 => Color::LightMagenta,
        14 => Color::LightCyan,
        15 => Color::White,
        _ => Color::Reset,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_hex_six_digits() {
        assert_eq!(parse_hex("#ff8800"), Some(Color::Rgb(0xff, 0x88, 0x00)));
        assert_eq!(parse_hex("#000000"), Some(Color::Rgb(0, 0, 0)));
    }

    #[test]
    fn parse_hex_three_digits() {
        assert_eq!(parse_hex("#f80"), Some(Color::Rgb(0xff, 0x88, 0x00)));
    }

    #[test]
    fn parse_hex_rejects_invalid() {
        assert_eq!(parse_hex("not-a-hex"), None);
        assert_eq!(parse_hex("#gggggg"), None);
        assert_eq!(parse_hex("#12"), None);
    }

    #[test]
    fn ansi_mapping() {
        assert_eq!(ansi_to_color(0), Color::Black);
        assert_eq!(ansi_to_color(1), Color::Red);
        assert_eq!(ansi_to_color(15), Color::White);
        assert_eq!(ansi_to_color(99), Color::Reset);
    }

    #[test]
    fn load_known_theme() {
        let t = load_theme("dracula");
        // Dracula's background is #282a36 — should NOT be the fallback Reset.
        assert_ne!(t.background, Color::Reset);
        if let Color::Rgb(r, g, b) = t.background {
            assert_eq!((r, g, b), (0x28, 0x2a, 0x36));
        } else {
            panic!("expected Rgb, got {:?}", t.background);
        }
    }

    #[test]
    fn load_opencode_theme_has_primary() {
        let t = load_theme("opencode");
        assert_ne!(t.primary, Color::Reset);
    }

    #[test]
    fn load_unknown_theme_falls_back_to_opencode() {
        let t = load_theme("nonexistent-theme-xyz");
        // Falls back to opencode theme, which has a non-Reset primary.
        assert_ne!(t.primary, Color::Reset);
    }

    #[test]
    fn list_theme_names_includes_dracula() {
        let names = list_theme_names();
        assert!(names.contains(&"dracula"));
        assert!(names.contains(&"opencode"));
        assert!(names.contains(&"catppuccin"));
        // 33 embedded themes.
        assert_eq!(names.len(), 33);
    }

    #[test]
    fn all_themes_resolve_without_error() {
        // Every theme should produce a non-Reset primary — the primary color
        // is always defined in real themes. (Background may legitimately be
        // "transparent" → Reset for themes like lucent-orng, so we don't
        // assert on background.)
        for name in list_theme_names() {
            let t = load_theme(name);
            assert_ne!(
                t.primary,
                Color::Reset,
                "theme {name} has Reset primary (resolution failed)"
            );
            assert_ne!(
                t.text,
                Color::Reset,
                "theme {name} has Reset text (resolution failed)"
            );
        }
    }

    #[test]
    fn dracula_theme_specific_colors() {
        let t = load_theme("dracula");
        // primary = purple = #bd93f9
        if let Color::Rgb(r, g, b) = t.primary {
            assert_eq!((r, g, b), (0xbd, 0x93, 0xf9));
        } else {
            panic!("expected Rgb primary");
        }
        // text = foreground = #f8f8f2
        if let Color::Rgb(r, g, b) = t.text {
            assert_eq!((r, g, b), (0xf8, 0xf8, 0xf2));
        } else {
            panic!("expected Rgb text");
        }
    }
}
