//! Authorization middleware — `packages/protocol/src/middleware/authorization.ts`
//!
//! In the TypeScript implementation, `Authorization` is an `HttpApiMiddleware`
//! that rejects unauthenticated requests with [`UnauthorizedError`]. The PTY
//! connect endpoint is exempted when a ticket query parameter is present.
//!
//! In Rust, this is expressed as an axum [`from_fn`] middleware that checks
//! for credentials and short-circuits with `401 Unauthorized` when missing.

use axum::extract::Request;
use axum::middleware::Next;
use axum::response::Response;

use crate::errors::ApiError;

/// Query parameter name carrying a PTY connect ticket.
///
/// Re-exported from the PTY group for middleware use.
pub const PTY_CONNECT_TICKET_QUERY: &str = "ticket";

/// Checks whether the request is a PTY connect upgrade with a ticket,
/// which bypasses normal credential checks.
///
/// The PTY connect handler is responsible for consuming and validating
/// the ticket itself.
pub fn has_pty_connect_ticket(uri: &axum::http::Uri) -> bool {
    let path = uri.path();
    let is_pty_connect = path.starts_with("/api/pty/")
        && path.ends_with("/connect")
        && path.matches('/').count() == 4;
    is_pty_connect && uri.query().map_or(false, |q| q.contains("ticket="))
}

/// Authorization middleware function.
///
/// Returns [`ApiError::Unauthorized`] when the request lacks valid
/// credentials. The actual credential validation logic is injected by the
/// server layer; this skeleton accepts all requests and serves as the
/// composition point.
pub async fn authorization(req: Request, next: Next) -> Result<Response, ApiError> {
    // PTY connect with a ticket bypasses authorization.
    if has_pty_connect_ticket(req.uri()) {
        return Ok(next.run(req).await);
    }

    // TODO: server layer injects credential validation here.
    // For now, allow all requests through.
    Ok(next.run(req).await)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_pty_connect_with_ticket() {
        let uri: axum::http::Uri =
            "/api/pty/abc-123/connect?ticket=xyz".parse().unwrap();
        assert!(has_pty_connect_ticket(&uri));
    }

    #[test]
    fn rejects_pty_connect_without_ticket() {
        let uri: axum::http::Uri = "/api/pty/abc-123/connect".parse().unwrap();
        assert!(!has_pty_connect_ticket(&uri));
    }

    #[test]
    fn rejects_non_pty_paths() {
        let uri: axum::http::Uri = "/api/session/abc/prompt?ticket=xyz".parse().unwrap();
        assert!(!has_pty_connect_ticket(&uri));
    }
}
