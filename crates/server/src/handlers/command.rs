use std::sync::Arc;
use axum::extract::State;
use axum::response::IntoResponse;
use axum::Json;
use opencode_core::state::AppState;

pub async fn list(State(_state): State<Arc<AppState>>) -> impl IntoResponse {
    Json(serde_json::json!({ "data": [] }))
}
