use std::sync::Arc;
use axum::extract::{Path, State};
use axum::response::IntoResponse;
use opencode_core::state::AppState;
use opencode_schema::ids::CredentialID;

pub async fn get(State(state): State<Arc<AppState>>, Path(id): Path<String>) -> impl IntoResponse {
    let cred_id = CredentialID::from(id);
    match state.credentials.get(&cred_id).await {
        Some(val) => axum::Json(serde_json::json!({ "data": val })).into_response().into_response(),
        None => (axum::http::StatusCode::NOT_FOUND, "Credential not found").into_response(),
    }
}

pub async fn delete(State(state): State<Arc<AppState>>, Path(id): Path<String>) -> impl IntoResponse {
    let cred_id = CredentialID::from(id);
    state.credentials.delete(&cred_id).await;
    (axum::http::StatusCode::OK, "Deleted")
}
