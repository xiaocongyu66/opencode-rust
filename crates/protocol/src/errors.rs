//! API error types with HTTP status code mapping.
//!
//! Each variant carries the data needed to render a JSON error body and the
//! matching HTTP status code, mirroring the TypeScript protocol error classes
//! defined in `packages/protocol/src/errors.ts`.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use thiserror::Error;

/// API errors mapped to HTTP status codes.
#[derive(Debug, Error)]
pub enum ApiError {
    /// 400 — The request was malformed or failed validation.
    #[error("{message}")]
    InvalidRequest {
        message: String,
        kind: Option<String>,
        field: Option<String>,
    },

    /// 401 — Authentication is required or has failed.
    #[error("{message}")]
    Unauthorized { message: String },

    /// 409 — The request conflicts with existing state.
    #[error("{message}")]
    Conflict { message: String, resource: Option<String> },

    /// 503 — The requested service is temporarily unavailable.
    #[error("{message}")]
    ServiceUnavailable { message: String, service: Option<String> },

    /// 500 — An unexpected internal error occurred.
    #[error("{message}")]
    Unknown { message: String, r#ref: Option<String> },

    /// 404 — The requested provider does not exist.
    #[error("{message}")]
    ProviderNotFound { provider_id: String, message: String },

    /// 404 — The requested session does not exist.
    #[error("{message}")]
    SessionNotFound { session_id: String, message: String },

    /// 404 — The requested message does not exist within the session.
    #[error("{message}")]
    MessageNotFound {
        session_id: String,
        message_id: String,
        message: String,
    },

    /// 400 — The pagination cursor is invalid.
    #[error("{message}")]
    InvalidCursor { message: String },

    /// 404 — The requested permission request does not exist.
    #[error("{message}")]
    PermissionNotFound { request_id: String, message: String },

    /// 404 — The requested question does not exist.
    #[error("{message}")]
    QuestionNotFound { request_id: String, message: String },

    /// 403 — The request is forbidden.
    #[error("{message}")]
    Forbidden { message: String },

    /// 404 — The requested PTY session does not exist.
    #[error("{message}")]
    PtyNotFound { pty_id: String, message: String },
}

impl ApiError {
    /// Returns the HTTP status code for this error.
    pub fn status(&self) -> StatusCode {
        match self {
            Self::InvalidRequest { .. } | Self::InvalidCursor { .. } => StatusCode::BAD_REQUEST,
            Self::Unauthorized { .. } => StatusCode::UNAUTHORIZED,
            Self::Forbidden { .. } => StatusCode::FORBIDDEN,
            Self::Conflict { .. } => StatusCode::CONFLICT,
            Self::ServiceUnavailable { .. } => StatusCode::SERVICE_UNAVAILABLE,
            Self::Unknown { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            Self::ProviderNotFound { .. }
            | Self::SessionNotFound { .. }
            | Self::MessageNotFound { .. }
            | Self::PermissionNotFound { .. }
            | Self::QuestionNotFound { .. }
            | Self::PtyNotFound { .. } => StatusCode::NOT_FOUND,
        }
    }

    /// Returns the error tag matching the TypeScript class name.
    pub fn tag(&self) -> &'static str {
        match self {
            Self::InvalidRequest { .. } => "InvalidRequestError",
            Self::Unauthorized { .. } => "UnauthorizedError",
            Self::Conflict { .. } => "ConflictError",
            Self::ServiceUnavailable { .. } => "ServiceUnavailableError",
            Self::Unknown { .. } => "UnknownError",
            Self::ProviderNotFound { .. } => "ProviderNotFoundError",
            Self::SessionNotFound { .. } => "SessionNotFoundError",
            Self::MessageNotFound { .. } => "MessageNotFoundError",
            Self::InvalidCursor { .. } => "InvalidCursorError",
            Self::PermissionNotFound { .. } => "PermissionNotFoundError",
            Self::QuestionNotFound { .. } => "QuestionNotFoundError",
            Self::Forbidden { .. } => "ForbiddenError",
            Self::PtyNotFound { .. } => "PtyNotFoundError",
        }
    }

    /// Builds the JSON body payload for this error.
    pub fn body(&self) -> serde_json::Value {
        match self {
            Self::InvalidRequest {
                message,
                kind,
                field,
            } => serde_json::json!({
                "error": self.tag(),
                "message": message,
                "kind": kind,
                "field": field,
            }),
            Self::Unauthorized { message } => serde_json::json!({
                "error": self.tag(),
                "message": message,
            }),
            Self::Conflict { message, resource } => serde_json::json!({
                "error": self.tag(),
                "message": message,
                "resource": resource,
            }),
            Self::ServiceUnavailable { message, service } => serde_json::json!({
                "error": self.tag(),
                "message": message,
                "service": service,
            }),
            Self::Unknown { message, r#ref } => serde_json::json!({
                "error": self.tag(),
                "message": message,
                "ref": r#ref,
            }),
            Self::ProviderNotFound {
                provider_id,
                message,
            } => serde_json::json!({
                "error": self.tag(),
                "providerID": provider_id,
                "message": message,
            }),
            Self::SessionNotFound {
                session_id,
                message,
            } => serde_json::json!({
                "error": self.tag(),
                "sessionID": session_id,
                "message": message,
            }),
            Self::MessageNotFound {
                session_id,
                message_id,
                message,
            } => serde_json::json!({
                "error": self.tag(),
                "sessionID": session_id,
                "messageID": message_id,
                "message": message,
            }),
            Self::InvalidCursor { message } => serde_json::json!({
                "error": self.tag(),
                "message": message,
            }),
            Self::PermissionNotFound {
                request_id,
                message,
            } => serde_json::json!({
                "error": self.tag(),
                "requestID": request_id,
                "message": message,
            }),
            Self::QuestionNotFound {
                request_id,
                message,
            } => serde_json::json!({
                "error": self.tag(),
                "requestID": request_id,
                "message": message,
            }),
            Self::Forbidden { message } => serde_json::json!({
                "error": self.tag(),
                "message": message,
            }),
            Self::PtyNotFound { pty_id, message } => serde_json::json!({
                "error": self.tag(),
                "ptyID": pty_id,
                "message": message,
            }),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.status(), axum::Json(self.body())).into_response()
    }
}

// ---------------------------------------------------------------------------
// Convenience constructors
// ---------------------------------------------------------------------------

impl ApiError {
    pub fn invalid_request(message: impl Into<String>) -> Self {
        Self::InvalidRequest {
            message: message.into(),
            kind: None,
            field: None,
        }
    }

    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self::Unauthorized {
            message: message.into(),
        }
    }

    pub fn forbidden(message: impl Into<String>) -> Self {
        Self::Forbidden {
            message: message.into(),
        }
    }

    pub fn conflict(message: impl Into<String>) -> Self {
        Self::Conflict {
            message: message.into(),
            resource: None,
        }
    }

    pub fn service_unavailable(message: impl Into<String>) -> Self {
        Self::ServiceUnavailable {
            message: message.into(),
            service: None,
        }
    }

    pub fn unknown(message: impl Into<String>) -> Self {
        Self::Unknown {
            message: message.into(),
            r#ref: None,
        }
    }

    pub fn invalid_cursor(message: impl Into<String>) -> Self {
        Self::InvalidCursor {
            message: message.into(),
        }
    }

    pub fn provider_not_found(provider_id: impl Into<String>, message: impl Into<String>) -> Self {
        Self::ProviderNotFound {
            provider_id: provider_id.into(),
            message: message.into(),
        }
    }

    pub fn session_not_found(session_id: impl Into<String>, message: impl Into<String>) -> Self {
        Self::SessionNotFound {
            session_id: session_id.into(),
            message: message.into(),
        }
    }

    pub fn message_not_found(
        session_id: impl Into<String>,
        message_id: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self::MessageNotFound {
            session_id: session_id.into(),
            message_id: message_id.into(),
            message: message.into(),
        }
    }

    pub fn permission_not_found(
        request_id: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self::PermissionNotFound {
            request_id: request_id.into(),
            message: message.into(),
        }
    }

    pub fn question_not_found(
        request_id: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self::QuestionNotFound {
            request_id: request_id.into(),
            message: message.into(),
        }
    }

    pub fn pty_not_found(pty_id: impl Into<String>, message: impl Into<String>) -> Self {
        Self::PtyNotFound {
            pty_id: pty_id.into(),
            message: message.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_codes_match_ts_definitions() {
        assert_eq!(ApiError::invalid_request("x").status(), StatusCode::BAD_REQUEST);
        assert_eq!(ApiError::unauthorized("x").status(), StatusCode::UNAUTHORIZED);
        assert_eq!(ApiError::forbidden("x").status(), StatusCode::FORBIDDEN);
        assert_eq!(ApiError::conflict("x").status(), StatusCode::CONFLICT);
        assert_eq!(
            ApiError::service_unavailable("x").status(),
            StatusCode::SERVICE_UNAVAILABLE
        );
        assert_eq!(ApiError::unknown("x").status(), StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(
            ApiError::provider_not_found("p", "x").status(),
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            ApiError::session_not_found("s", "x").status(),
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            ApiError::invalid_cursor("x").status(),
            StatusCode::BAD_REQUEST
        );
    }

    #[test]
    fn body_includes_error_tag() {
        let body = ApiError::session_not_found("s1", "missing").body();
        assert_eq!(body["error"], "SessionNotFoundError");
        assert_eq!(body["sessionID"], "s1");
        assert_eq!(body["message"], "missing");
    }
}
