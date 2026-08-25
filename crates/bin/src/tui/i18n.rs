//! Simple i18n — 不依赖 rust-i18n crate，直接用 include_str! 嵌入翻译

use std::collections::HashMap;
use std::sync::OnceLock;

static EN_YAML: &str = include_str!("../../locales/en.yml");
static ZH_YAML: &str = include_str!("../../locales/zh.yml");

static TRANSLATIONS: OnceLock<HashMap<String, String>> = OnceLock::new();

fn parse_yaml_flat(content: &str, lang: &str) -> HashMap<String, String> {
    let mut result = HashMap::new();
    let prefix = format!("{}:", lang);
    let mut in_lang = false;

    for line in content.lines() {
        if line.trim_start().starts_with('#') || line.trim().is_empty() {
            continue;
        }
        if line.starts_with(&prefix) {
            in_lang = true;
            continue;
        }
        // Check if this is a new top-level key (no leading spaces)
        if !line.starts_with(' ') && !line.starts_with('\t') && line.ends_with(':') {
            in_lang = false;
            continue;
        }
        if !in_lang {
            continue;
        }
        // Parse "  key: value" format
        let trimmed = line.trim();
        if let Some(colon_pos) = trimmed.find(':') {
            let key = trimmed[..colon_pos].trim();
            let value = trimmed[colon_pos + 1..].trim();
            // Remove surrounding quotes
            let value = value.trim_matches('"');
            if !key.is_empty() {
                result.insert(key.to_string(), value.to_string());
            }
        }
    }
    result
}

fn get_translations() -> &'static HashMap<String, String> {
    TRANSLATIONS.get_or_init(|| {
        let locale = current_locale();
        match locale.as_str() {
            "zh" => parse_yaml_flat(ZH_YAML, "zh"),
            _ => parse_yaml_flat(EN_YAML, "en"),
        }
    })
}

fn current_locale() -> String {
    for key in &["LC_ALL", "LC_MESSAGES", "LANG"] {
        if let Ok(val) = std::env::var(key) {
            let lower = val.to_lowercase();
            if lower.starts_with("zh") {
                return "zh".to_string();
            }
            if !lower.is_empty() && lower != "c" && lower != "posix" {
                return "en".to_string();
            }
        }
    }
    "en".to_string()
}

/// Translate a key to the current locale.
pub fn t(key: &str) -> String {
    let translations = get_translations();
    translations.get(key).cloned().unwrap_or_else(|| key.to_string())
}

/// Translate with format arguments.
pub fn tf(key: &str, args: &[(&str, &str)]) -> String {
    let mut result = t(key);
    for (k, v) in args {
        result = result.replace(&format!("%{{{}}}", k), v);
    }
    result
}

/// Initialize i18n (forces re-reading locale)
pub fn init() {
    // TRANSLATIONS is OnceLock, so it will be initialized on first use
    // This function just forces the locale detection
}

/// Set locale manually
pub fn set_locale(_locale: &str) {
    // In this simple implementation, locale is detected from environment
}

/// Macro for convenient translation.
///
/// Two forms:
///   t!("key")                              → simple lookup
///   t!("key", name = value, name2 = value2) → with named args, %{name} replaced
#[macro_export]
macro_rules! t {
    ($k:expr) => {
        $crate::tui::i18n::t($k)
    };
    ($k:expr, $($name:ident = $val:expr),+ $(,)?) => {{
        let pairs: Vec<(String, String)> = vec![
            $((stringify!($name).to_string(), ($val).to_string())),+
        ];
        let args_ref: Vec<(&str, &str)> =
            pairs.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
        $crate::tui::i18n::tf($k, &args_ref)
    }};
}
