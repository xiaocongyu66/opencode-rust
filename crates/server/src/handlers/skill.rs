use std::sync::Arc;
use axum::extract::State;
use axum::response::IntoResponse;
use axum::Json;
use opencode_core::state::AppState;

pub async fn list(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let skills = state.skills.list().await;
    Json(serde_json::json!({ "data": skills }))
}
