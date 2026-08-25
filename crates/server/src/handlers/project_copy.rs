//! Project copy handlers.

use std::sync::Arc;
use axum::extract::{Path, State};
use axum::response::IntoResponse;
use axum::Json;
use opencode_core::state::AppState;

pub async fn create(State(_state): State<Arc<AppState>>, Path(_project_id): Path<String>, Json(_body): Json<serde_json::Value>) -> impl IntoResponse {
    (axum::http::StatusCode::CREATED, "Project copy not yet implemented")
}

pub async fn delete(State(_state): State<Arc<AppState>>, Path(_project_id): Path<String>) -> impl IntoResponse {
    (axum::http::StatusCode::OK, "Deleted")
}

pub async fn refresh(State(_state): State<Arc<AppState>>, Path(_project_id): Path<String>) -> impl IntoResponse {
    (axum::http::StatusCode::OK, "Refresh not yet implemented")
}
