//! Built-in tool implementations (60 tools).
//!
//! Aligned with the official Claude Code tool set.
//! Each tool is a flat module file implementing the `Tool` trait.

pub mod tool;
pub mod registry;

// Core file/shell tools (9)
pub mod bash;
pub mod edit;
pub mod read;
pub mod write;
pub mod glob;
pub mod grep;
pub mod webfetch;
pub mod websearch;
pub mod todowrite;

// Task system (6) + shared store
pub mod task;
pub mod task_create;
pub mod task_get;
pub mod task_list;
pub mod task_output;
pub mod task_stop;
pub mod task_update;

// Notebook + shell variants
pub mod notebook_edit;
pub mod powershell;

// Planning
pub mod enter_plan_mode;
pub mod exit_plan_mode;
pub mod verify_plan_execution;

// Agent + skills + questions
pub mod agent;
pub mod skill;
pub mod ask_user_question;
pub mod discover_skills;

// MCP
pub mod mcp;
pub mod mcp_auth;
pub mod list_mcp_resources;
pub mod read_mcp_resource;

// Cron + worktree + memory
pub mod schedule_cron;
pub mod enter_worktree;
pub mod exit_worktree;
pub mod local_memory_recall;

// Monitoring + notifications
pub mod monitor;
pub mod push_notification;
pub mod remote_trigger;
pub mod review_artifact;

// Communication
pub mod send_message;
pub mod send_user_file;

// Code/utility
pub mod snip;
pub mod subscribe_pr;
pub mod suggest_background_pr;
pub mod terminal_capture;
pub mod artifact;

// Teams
pub mod team;

// Misc tools
pub mod ctx_inspect;
pub mod goal;
pub mod list_peers;
pub mod brief;
pub mod execute;
pub mod repl;
pub mod web_browser;
pub mod vault_http_fetch;
pub mod search_extra_tools;
pub mod execute_extra_tool;
pub mod synthetic_output;
pub mod tungsten;
pub mod overflow_test;

// Config + LSP + sleep
pub mod config;
pub mod lsp;
pub mod sleep;
