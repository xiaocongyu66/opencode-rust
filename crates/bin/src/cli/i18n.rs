//! i18n support for CLI.
//!
//! CLI 和 TUI 共用同一套 i18n 实现(见 `crate::tui::i18n`)。
//! 翻译文案通过 `t!` 宏访问,locale 从环境变量(LC_ALL / LC_MESSAGES / LANG)自动检测。

pub fn set_locale(locale: &str) {
    crate::tui::i18n::set_locale(locale);
}

pub fn init() {
    crate::tui::i18n::init();
}

#[allow(dead_code)]
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
