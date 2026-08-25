//! opencode-cli crate
//!
//! Command-line interface entry point for opencode.

rust_i18n::i18n!("locales", fallback = "en");

pub mod i18n;
pub mod cli;
pub mod commands;
