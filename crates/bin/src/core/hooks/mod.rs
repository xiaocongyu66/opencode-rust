//! Hook system — lifecycle extension points (claude-code-book Ch08).
//!
//! Five hook types share a JSON protocol: Command (shell), Prompt (LLM),
//! Agent (multi-step LLM), HTTP (POST), Function (in-process). For P2
//! we implement Command fully and stub the rest.
//!
//! 26 lifecycle events: SessionStart, SessionEnd, UserPromptSubmit,
//! PreToolUse, PostToolUse, PreCompact, PostCompact, Notification, Stop,
//! SubagentStop, PreFileEdit, ...
//!
//! Six-layer priority: plugin < user < project < local < flag < policy.
//! First non-passthrough decision wins.

pub mod executor;
pub mod protocol;
pub mod registry;

pub use executor::{run_chain, run_hook};
pub use protocol::{
    ALL_EVENTS, EVENT_NOTIFICATION, EVENT_POST_COMPACT, EVENT_POST_TOOL_USE,
    EVENT_PRE_COMPACT, EVENT_PRE_FILE_EDIT, EVENT_PRE_TOOL_USE, EVENT_SESSION_END,
    EVENT_SESSION_START, EVENT_STOP, EVENT_SUBAGENT_STOP, EVENT_USER_PROMPT_SUBMIT,
    HookDecision, HookInput, HookOutput,
};
pub use registry::{HookConfig, HookEntry, HookLayer, HookRegistry};
