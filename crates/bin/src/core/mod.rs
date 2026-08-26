//! opencode-core crate
//!
//! Core business logic: sessions, agents, config, and orchestration.
//! Ported from `packages/opencode/src/` in the TS original.

pub mod config;
pub mod session;
pub mod agent;
pub mod model;
pub mod provider;
pub mod project;
pub mod credential;
pub mod filesystem;
pub mod git;
pub mod permission;
pub mod integration;
pub mod skill;
pub mod event;
pub mod process;
pub mod shell;
pub mod state;
pub mod file;
pub mod policy;
pub mod snapshot;
pub mod workspace;
pub mod instruction_context;
pub mod system_context;
pub mod location;
pub mod observability;
pub mod repository;
pub mod background_job;
pub mod tool_output_store;
pub mod patch;
pub mod hooks;
pub mod acp;

// New modules ported from TS original
pub mod auth;
pub mod bus;
pub mod storage;
pub mod sync;
pub mod ide;
pub mod mcp;
pub mod lsp;
pub mod rsopencode;
