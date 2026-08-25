//! Project copy routes — `packages/protocol/src/groups/project-copy.ts`
//!
//! Project copy routes manage project copies under the `/experimental`
//! prefix. The `ProjectCopyError` is defined inline in the TS group rather
//! than in `errors.ts`, so it is mirrored here.

use axum::Router;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use thiserror::Error;

use crate::api::ApiGroup;

/// Root path for project copy routes.
pub const PROJECT_COPY_ROOT: &str = "/experimental/project/:projectID/copy";

/// `POST` — Create a project copy.
pub const PROJECT_COPY_CREATE: &str = PROJECT_COPY_ROOT;
/// `DELETE` — Remove a project copy.
pub const PROJECT_COPY_REMOVE: &str = PROJECT_COPY_ROOT;
/// `POST` — Refresh a project copy.
pub const PROJECT_COPY_REFRESH: &str = "/experimental/project/:projectID/copy/refresh";

/// 400 — A project copy error occurred.
///
/// Mirrors the inline `ProjectCopyError` from `project-copy.ts`.
#[derive(Debug, Error, Serialize)]
#[error("{message}")]
pub struct ProjectCopyError {
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub force_required: Option<bool>,
}

impl IntoResponse for ProjectCopyError {
    fn into_response(self) -> Response {
        (
            StatusCode::BAD_REQUEST,
            axum::Json(serde_json::json!({
                "error": "ProjectCopyError",
                "data": {
                    "message": self.message,
                    "forceRequired": self.force_required,
                }
            })),
        )
            .into_response()
    }
}

/// Project copy API group.
pub struct ProjectCopyGroup;

impl ApiGroup for ProjectCopyGroup {
    type State = ();
    fn routes() -> Router<()> {
        Router::new()
    }
}
