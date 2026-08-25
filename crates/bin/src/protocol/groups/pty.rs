//! PTY routes — `packages/protocol/src/groups/pty.ts`
//!
//! PTY routes manage pseudo-terminal sessions. The connect endpoint upgrades
//! to a WebSocket connection using a short-lived single-use ticket.

use axum::Router;

use crate::protocol::api::ApiGroup;

// ---------------------------------------------------------------------------
// Route paths
// ---------------------------------------------------------------------------

/// `GET` — List PTY sessions for a location.
pub const PTY_LIST: &str = "/api/pty";
/// `POST` — Create a PTY session for a location.
pub const PTY_CREATE: &str = "/api/pty";
/// `GET` — Get one PTY session by ID.
pub const PTY_GET: &str = "/api/pty/:ptyID";
/// `PUT` — Update the title or viewport size of a PTY session.
pub const PTY_UPDATE: &str = "/api/pty/:ptyID";
/// `DELETE` — Terminate and remove one PTY session.
pub const PTY_REMOVE: &str = "/api/pty/:ptyID";
/// `POST` — Create a short-lived single-use WebSocket connection ticket.
pub const PTY_CONNECT_TOKEN: &str = "/api/pty/:ptyID/connect-token";
/// `GET` — Establish a WebSocket connection (upgrades from HTTP).
pub const PTY_CONNECT: &str = "/api/pty/:ptyID/connect";

// ---------------------------------------------------------------------------
// Ticket query / header constants
// ---------------------------------------------------------------------------

/// Query parameter name carrying the PTY connect ticket.
pub const PTY_CONNECT_TICKET_QUERY: &str = "ticket";
/// Header name signalling an authorized PTY connect upgrade.
pub const PTY_CONNECT_TOKEN_HEADER: &str = "x-opencode-ticket";
/// Header value expected alongside [`PTY_CONNECT_TOKEN_HEADER`].
pub const PTY_CONNECT_TOKEN_HEADER_VALUE: &str = "1";

/// PTY API group.
pub struct PtyGroup;

impl ApiGroup for PtyGroup {
    type State = ();
    fn routes() -> Router<()> {
        Router::new()
    }
}
