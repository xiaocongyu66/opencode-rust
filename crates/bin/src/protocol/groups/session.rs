//! Session routes — `packages/protocol/src/groups/session.ts`
//!
//! The session group is the largest API surface, covering session lifecycle,
//! agent/model switching, prompt admission, compaction, revert, context,
//! history, event streaming, interruption, and message retrieval.

use axum::Router;

use crate::protocol::api::ApiGroup;

// ---------------------------------------------------------------------------
// Route paths
// ---------------------------------------------------------------------------

/// `GET` — List sessions.
pub const SESSION_LIST: &str = "/api/session";
/// `POST` — Create a session.
pub const SESSION_CREATE: &str = "/api/session";
/// `GET` — List active (running) sessions.
pub const SESSION_ACTIVE: &str = "/api/session/active";
/// `GET` — Get a session by ID.
pub const SESSION_GET: &str = "/api/session/:sessionID";
/// `POST` — Switch the session's agent.
pub const SESSION_SWITCH_AGENT: &str = "/api/session/:sessionID/agent";
/// `POST` — Switch the session's model.
pub const SESSION_SWITCH_MODEL: &str = "/api/session/:sessionID/model";
/// `POST` — Admit a prompt and schedule agent-loop execution.
pub const SESSION_PROMPT: &str = "/api/session/:sessionID/prompt";
/// `POST` — Compact the session conversation.
pub const SESSION_COMPACT: &str = "/api/session/:sessionID/compact";
/// `POST` — Wait for the session agent loop to become idle.
pub const SESSION_WAIT: &str = "/api/session/:sessionID/wait";
/// `POST` — Stage a session revert boundary.
pub const SESSION_REVERT_STAGE: &str = "/api/session/:sessionID/revert/stage";
/// `POST` — Clear a staged revert.
pub const SESSION_REVERT_CLEAR: &str = "/api/session/:sessionID/revert/clear";
/// `POST` — Commit a staged revert.
pub const SESSION_REVERT_COMMIT: &str = "/api/session/:sessionID/revert/commit";
/// `GET` — Get the active context messages for a session.
pub const SESSION_CONTEXT: &str = "/api/session/:sessionID/context";
/// `GET` — Get one page of durable session history events.
pub const SESSION_HISTORY: &str = "/api/session/:sessionID/history";
/// `GET` — Subscribe to durable session events (SSE stream).
pub const SESSION_EVENTS: &str = "/api/session/:sessionID/event";
/// `POST` — Interrupt active session execution.
pub const SESSION_INTERRUPT: &str = "/api/session/:sessionID/interrupt";
/// `GET` — Get a specific message within a session.
pub const SESSION_MESSAGE: &str = "/api/session/:sessionID/message/:messageID";

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

/// Session API group.
///
/// Skeleton implementation — handler registration is deferred until the
/// server layer provides concrete service bindings.
pub struct SessionGroup;

impl ApiGroup for SessionGroup {
    type State = ();
    fn routes() -> Router<()> {
        Router::new()
    }
}
