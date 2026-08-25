use std::sync::Arc;
use axum::extract::State;
use axum::response::IntoResponse;
use crate::core::state::AppState;

pub async fn subscribe(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let _rx = state.events.subscribe();
    axum::response::Response::builder()
        .header("content-type", "text/event-stream")
        .header("cache-control", "no-cache")
        .body(axum::body::Body::empty())
        .unwrap()
}
