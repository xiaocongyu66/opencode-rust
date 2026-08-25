//! Project copy handlers.

use std::sync::Arc;
use axum::extract::{Path, State};
use axum::response::IntoResponse;
use axum::Json;
use crate::core::state::AppState;

pub async fn create(State(_state): State<Arc<AppState>>, Path(_project_id): Path<String>, Json(_body): Json<serde_json::Value>) -> impl IntoResponse {
    Json(serde_json::json!({ "data": {} }))
}

pub async fn delete(State(_state): State<Arc<AppState>>, Path(_project_id): Path<String>) -> impl IntoResponse {
    Json(serde_json::json!({ "data": null }))
}

pub async fn refresh(State(_state): State<Arc<AppState>>, Path(_project_id): Path<String>) -> impl IntoResponse {
    Json(serde_json::json!({ "data": {} }))
}
