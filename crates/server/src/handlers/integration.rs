use std::sync::Arc;
use axum::extract::{Path, State};
use axum::response::IntoResponse;
use axum::Json;
use opencode_core::state::AppState;

pub async fn list(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let integrations = state.integrations.list().await;
    Json(serde_json::json!({ "data": integrations }))
}

pub async fn get(State(_state): State<Arc<AppState>>, Path(_id): Path<String>) -> impl IntoResponse {
    Json(serde_json::json!({ "data": serde_json::Value::Null })).into_response()
}

pub async fn connect_oauth(State(_state): State<Arc<AppState>>, Path(_id): Path<String>, Json(_body): Json<serde_json::Value>) -> impl IntoResponse {
    (axum::http::StatusCode::OK, "OAuth not yet implemented")
}

pub async fn connect_key(State(_state): State<Arc<AppState>>, Path(_id): Path<String>, Json(_body): Json<serde_json::Value>) -> impl IntoResponse {
    (axum::http::StatusCode::OK, "Key auth not yet implemented")
}

pub async fn get_attempt(State(_state): State<Arc<AppState>>, Path(_attempt_id): Path<String>) -> impl IntoResponse {
    (axum::http::StatusCode::NOT_FOUND, "Attempt not found")
}

pub async fn complete_attempt(State(_state): State<Arc<AppState>>, Path(_attempt_id): Path<String>, Json(_body): Json<serde_json::Value>) -> impl IntoResponse {
    (axum::http::StatusCode::OK, "Attempt completed")
}
