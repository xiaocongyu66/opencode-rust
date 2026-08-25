//! Provider handlers.

use std::sync::Arc;
use axum::extract::State;
use axum::response::IntoResponse;
use axum::Json;
use crate::core::state::AppState;

pub async fn list(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let providers = state.providers.list().await;
    Json(serde_json::json!({ "data": providers }))
}
