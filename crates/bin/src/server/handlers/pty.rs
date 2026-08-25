use std::sync::Arc;
use axum::extract::{Path, State};
use axum::response::IntoResponse;
use axum::Json;
use crate::core::state::AppState;

pub async fn list(State(_state): State<Arc<AppState>>) -> impl IntoResponse {
    Json(serde_json::json!({ "data": [] }))
}

pub async fn create(State(_state): State<Arc<AppState>>, Json(_body): Json<serde_json::Value>) -> impl IntoResponse {
    Json(serde_json::json!({
        "data": {
            "id": "pty_0",
            "title": "",
            "command": "",
            "args": [],
            "cwd": "",
            "status": "running",
            "pid": 0
        }
    }))
}

pub async fn get(State(_state): State<Arc<AppState>>, Path(_id): Path<String>) -> impl IntoResponse {
    (axum::http::StatusCode::NOT_FOUND, "PTY not found")
}

pub async fn connect(State(_state): State<Arc<AppState>>, Path(_id): Path<String>) -> impl IntoResponse {
    Json(serde_json::json!({ "data": null }))
}

pub async fn connect_token(State(_state): State<Arc<AppState>>, Path(_id): Path<String>) -> impl IntoResponse {
    Json(serde_json::json!({ "ticket": "not-implemented", "expires_in": 0 }))
}
