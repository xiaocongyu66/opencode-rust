//! Tool handlers.

use std::sync::Arc;
use axum::extract::State;
use axum::response::IntoResponse;
use axum::Json;
use opencode_core::state::AppState;
use opencode_tools::registry::ToolRegistry;

pub async fn list(State(_state): State<Arc<AppState>>) -> impl IntoResponse {
    let registry = ToolRegistry::builtin();
    let defs = registry.definitions();
    Json(serde_json::json!({ "data": defs }))
}
