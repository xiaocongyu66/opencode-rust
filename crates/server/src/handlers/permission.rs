use std::sync::Arc;
use axum::extract::{Path, State};
use axum::response::IntoResponse;
use axum::Json;
use opencode_core::state::AppState;

pub async fn list_requests(State(_state): State<Arc<AppState>>) -> impl IntoResponse {
    Json(serde_json::json!({ "data": [] }))
}

pub async fn list_saved(State(_state): State<Arc<AppState>>) -> impl IntoResponse {
    Json(serde_json::json!({ "data": [] }))
}

pub async fn save(State(_state): State<Arc<AppState>>, Json(_body): Json<serde_json::Value>) -> impl IntoResponse {
    (axum::http::StatusCode::CREATED, "Saved")
}

pub async fn get_saved(State(_state): State<Arc<AppState>>, Path(_id): Path<String>) -> impl IntoResponse {
    (axum::http::StatusCode::NOT_FOUND, "Not found")
}

pub async fn delete_saved(State(_state): State<Arc<AppState>>, Path(_id): Path<String>) -> impl IntoResponse {
    (axum::http::StatusCode::OK, "Deleted")
}

pub async fn session_list(State(_state): State<Arc<AppState>>, Path(_session_id): Path<String>) -> impl IntoResponse {
    Json(serde_json::json!({ "data": [] }))
}

pub async fn get_request(State(_state): State<Arc<AppState>>, Path((_session_id, _request_id)): Path<(String, String)>) -> impl IntoResponse {
    (axum::http::StatusCode::NOT_FOUND, "Not found")
}

pub async fn reply(State(_state): State<Arc<AppState>>, Path((_session_id, _request_id)): Path<(String, String)>, Json(_body): Json<serde_json::Value>) -> impl IntoResponse {
    (axum::http::StatusCode::OK, "Reply sent")
}
