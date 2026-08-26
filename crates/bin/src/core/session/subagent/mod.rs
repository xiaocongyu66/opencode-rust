//! Sub-agent system — Fork, bg agents, definitions (claude-code-book Ch09).
//!
//! Three sources of sub-agents (built-in / plugin / user Markdown) with
//! Fork byte-level context inheritance and background agent lifecycle.

pub mod bg_agent;
pub mod definition;
pub mod fork;

pub use bg_agent::{all, clear, is_background, register, unregister};
pub use definition::{builtin_agents, BaseAgentDefinition};
pub use fork::{append_fork_marker, build_fork_prefix, ForkPrefix};
