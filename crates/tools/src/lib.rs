//! opencode-tools crate
//!
//! Built-in tool implementations exposed to LLM agents.

pub mod tool;
pub mod registry;
pub mod bash;
pub mod edit;
pub mod read;
pub mod write;
pub mod glob;
pub mod grep;
pub mod webfetch;
pub mod websearch;
pub mod todowrite;
pub mod question;
pub mod skill;
pub mod apply_patch;
pub mod task;
pub mod plan;
pub mod code_search;
pub mod lsp;
pub mod shell;
pub mod truncate;
pub mod external_directory;
