//! opencode-tui crate
//!
//! Terminal user interface for opencode.

rust_i18n::i18n!("locales", fallback = "en");

pub mod i18n;
pub mod app;
pub mod ui;
pub mod event;
