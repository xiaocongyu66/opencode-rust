use std::sync::Arc;
use axum::extract::{Path, State};
use axum::response::IntoResponse;
use axum::Json;
use crate::core::state::AppState;

pub async fn list_requests(State(_state): State<Arc<AppState>>) -> impl IntoResponse {
    Json(serde_json::json!({ "data": [] }))
}

pub async fn session_list(State(_state): State<Arc<AppState>>, Path(_session_id): Path<String>) -> impl IntoResponse {
    Json(serde_json::json!({ "data": [] }))
}

pub async fn reply(State(_state): State<Arc<AppState>>, Path((_session_id, _request_id)): Path<(String, String)>, Json(_body): Json<serde_json::Value>) -> impl IntoResponse {
    (axum::http::StatusCode::OK, "Reply sent")
}

pub async fn reject(State(_state): State<Arc<AppState>>, Path((_session_id, _request_id)): Path<(String, String)>, Json(_body): Json<serde_json::Value>) -> impl IntoResponse {
    (axum::http::StatusCode::OK, "Rejected")
}
