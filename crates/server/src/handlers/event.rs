use std::sync::Arc;
use axum::extract::State;
use axum::response::IntoResponse;
use opencode_core::state::AppState;

pub async fn subscribe(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let _rx = state.events.subscribe();
    // Return initial acknowledgment; full SSE streaming requires a dedicated handler
    (axum::http::StatusCode::OK, "Event streaming not yet implemented")
}
