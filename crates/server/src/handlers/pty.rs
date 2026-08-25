use std::sync::Arc;
use axum::extract::{Path, State};
use axum::response::IntoResponse;
use axum::Json;
use opencode_core::state::AppState;

pub async fn list(State(_state): State<Arc<AppState>>) -> impl IntoResponse {
    Json(serde_json::json!({ "data": [] }))
}

pub async fn create(State(_state): State<Arc<AppState>>, Json(_body): Json<serde_json::Value>) -> impl IntoResponse {
    (axum::http::StatusCode::CREATED, "PTY creation not yet implemented")
}

pub async fn get(State(_state): State<Arc<AppState>>, Path(_id): Path<String>) -> impl IntoResponse {
    (axum::http::StatusCode::NOT_FOUND, "PTY not found")
}

pub async fn connect(State(_state): State<Arc<AppState>>, Path(_id): Path<String>) -> impl IntoResponse {
    (axum::http::StatusCode::OK, "PTY WebSocket not yet implemented")
}

pub async fn connect_token(State(_state): State<Arc<AppState>>, Path(_id): Path<String>) -> impl IntoResponse {
    Json(serde_json::json!({ "ticket": "not-implemented", "expires_in": 0 }))
}
