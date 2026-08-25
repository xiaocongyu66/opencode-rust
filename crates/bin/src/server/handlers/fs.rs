use std::sync::Arc;
use axum::extract::{Path, State};
use axum::response::IntoResponse;
use axum::Json;
use crate::core::state::AppState;
use crate::schema::common::AbsolutePath;

pub async fn list(State(_state): State<Arc<AppState>>, Json(body): Json<serde_json::Value>) -> impl IntoResponse {
    let dir = body.get("directory").and_then(|v| v.as_str()).unwrap_or(".");
    match crate::core::filesystem::FileSystem::list_dir(&AbsolutePath::new(dir)).await {
        Ok(entries) => Json(serde_json::json!({ "data": entries })).into_response(),
        Err(e) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, format!("Error: {}", e)).into_response(),
    }
}

pub async fn find(State(_state): State<Arc<AppState>>, Json(_body): Json<serde_json::Value>) -> impl IntoResponse {
    Json(serde_json::json!({ "data": [] }))
}

pub async fn read(State(_state): State<Arc<AppState>>, Path(path): Path<String>) -> impl IntoResponse {
    match tokio::fs::read_to_string(&path).await {
        Ok(content) => Json(serde_json::json!({ "data": content })).into_response(),
        Err(e) => (axum::http::StatusCode::NOT_FOUND, format!("File not found: {}", e)).into_response(),
    }
}
