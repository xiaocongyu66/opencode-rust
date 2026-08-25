//! i18n support for opencode-tui.

/// Set the current locale at runtime.
pub fn set_locale(locale: &str) {
    rust_i18n::set_locale(locale);
}

/// Detect the system locale and apply it.
pub fn init() {
    let locale = detect_system_locale();
    set_locale(&locale);
}

fn detect_system_locale() -> String {
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
