//! Question routes — `packages/protocol/src/groups/question.ts`
//!
//! Question endpoints are split between location-scoped routes (request list)
//! and session-scoped routes (list, reply, reject) that require session
//! placement middleware.

use axum::Router;

use crate::protocol::api::ApiGroup;

// ---------------------------------------------------------------------------
// Location-scoped routes
// ---------------------------------------------------------------------------

/// `GET` — List pending question requests for a location.
pub const QUESTION_REQUEST_LIST: &str = "/api/question/request";

// ---------------------------------------------------------------------------
// Session-scoped routes
// ---------------------------------------------------------------------------

/// `GET` — List pending question requests owned by a session.
pub const SESSION_QUESTION_LIST: &str = "/api/session/:sessionID/question";
/// `POST` — Answer a pending question request.
pub const SESSION_QUESTION_REPLY: &str = "/api/session/:sessionID/question/:requestID/reply";
/// `POST` — Reject a pending question request.
pub const SESSION_QUESTION_REJECT: &str = "/api/session/:sessionID/question/:requestID/reject";

/// Question API group.
pub struct QuestionGroup;

impl ApiGroup for QuestionGroup {
    type State = ();
    fn routes() -> Router<()> {
        Router::new()
    }
}
