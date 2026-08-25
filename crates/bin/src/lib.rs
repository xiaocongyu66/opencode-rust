//! rsopencode — AI-powered development tool (Rust port of opencode)

#![allow(dead_code)]

pub mod schema;
pub mod llm;
pub mod tools;
pub mod core;
pub mod server;
pub mod protocol;
pub mod cli;
pub mod tui;

pub use tui::i18n;

#[cfg(test)]
mod tests {
    use crate::tui::i18n::{t, tf};

    #[test]
    fn t_returns_translation_for_known_key() {
        // 已知 key 不应原样返回(说明命中了翻译表)
        let v = t("tui.message.user_prefix");
        assert_ne!(v, "tui.message.user_prefix");
        assert!(!v.is_empty());
    }

    #[test]
    fn t_returns_key_for_missing_key() {
        // 未知 key 应原样返回(回退到 key 本身)
        let v = t("definitely.nonexistent.key.123");
        assert_eq!(v, "definitely.nonexistent.key.123");
    }

    #[test]
    fn tf_replaces_named_placeholders() {
        // %{name} 占位符应被替换
        let v = tf("tui.message.tool", &[("name", "Bash")]);
        assert!(v.contains("Bash"), "expected Bash in: {v}");
        assert!(!v.contains("%{name}"), "unresolved placeholder in: {v}");
    }

    #[test]
    fn tf_replaces_multiple_placeholders() {
        let v = tf("tui.message.tool_failed", &[("id", "call_42"), ("error", "boom")]);
        assert!(v.contains("call_42"));
        assert!(v.contains("boom"));
        assert!(!v.contains("%{id}"));
        assert!(!v.contains("%{error}"));
    }

    #[test]
    fn t_macro_simple_form() {
        let v: String = crate::t!("tui.message.user_prefix");
        assert!(!v.is_empty());
    }

    #[test]
    fn t_macro_named_args_form() {
        let v: String = crate::t!("tui.message.tool", name = "Bash");
        assert!(v.contains("Bash"));
        assert!(!v.contains("%{name}"));
    }

    #[test]
    fn t_macro_multiple_named_args() {
        let v: String =
            crate::t!("tui.message.tool_failed", id = "call_1", error = "oops");
        assert!(v.contains("call_1"));
        assert!(v.contains("oops"));
    }
}
