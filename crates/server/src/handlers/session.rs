//! Session handlers.

use std::sync::Arc;
use axum::extract::{Path, Query, State};
use axum::response::IntoResponse;
use axum::Json;
use opencode_core::state::AppState;
use opencode_schema::ids::{AgentID, SessionID};
use opencode_schema::prompt::Prompt;
use opencode_schema::session::{SessionDelivery, SessionInfo};
use serde::Deserialize;

pub async fn list(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let sessions = state.sessions.list(50, 0).await;
    Json(serde_json::json!({ "data": sessions }))
}

pub async fn create(
    State(state): State<Arc<AppState>>,
    Json(body): Json<CreateSessionBody>,
) -> impl IntoResponse {
    let now = chrono::Utc::now();
    let info = SessionInfo {
        id: body.id.unwrap_or_else(SessionID::new),
        parent_id: None,
        project_id: opencode_schema::ids::ProjectID::global(),
        agent: body.agent,
        model: body.model,
        cost: 0.0,
        tokens: Default::default(),
        time: opencode_schema::session::SessionTime {
            created: now,
            updated: now,
            archived: None,
        },
        title: "New session".to_string(),
        location: body.location.unwrap_or(opencode_schema::location::LocationRef {
            directory: opencode_schema::common::AbsolutePath::new(
                std::env::current_dir().map(|p| p.to_string_lossy().to_string()).unwrap_or_default()
            ),
            workspace_id: None,
        }),
        subpath: None,
        revert: None,
    };
    let created = state.sessions.create(info).await;
    Json(serde_json::json!({ "data": created }))
}

pub async fn list_active(
    State(_state): State<Arc<AppState>>,
) -> impl IntoResponse {
    Json(serde_json::json!({ "data": {} }))
}

pub async fn get(
    State(state): State<Arc<AppState>>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    let id = SessionID::from_str(&session_id);
    match state.sessions.get(&id).await {
        Some(info) => Json(serde_json::json!({ "data": info })).into_response(),
        None => (axum::http::StatusCode::NOT_FOUND, "Session not found").into_response(),
    }
}

pub async fn switch_agent(
    State(state): State<Arc<AppState>>,
    Path(session_id): Path<String>,
    Json(body): Json<SwitchAgentBody>,
) -> impl IntoResponse {
    let id = SessionID::from_str(&session_id);
    if let Some(mut info) = state.sessions.get(&id).await {
        info.agent = Some(body.agent);
        let updated = state.sessions.update(info).await;
        Json(serde_json::json!({ "data": updated })).into_response()
    } else {
        (axum::http::StatusCode::NOT_FOUND, "Session not found").into_response()
    }
}

pub async fn switch_model(
    State(state): State<Arc<AppState>>,
    Path(session_id): Path<String>,
    Json(body): Json<SwitchModelBody>,
) -> impl IntoResponse {
    let id = SessionID::from_str(&session_id);
    if let Some(mut info) = state.sessions.get(&id).await {
        info.model = Some(body.model);
        let updated = state.sessions.update(info).await;
        Json(serde_json::json!({ "data": updated })).into_response()
    } else {
        (axum::http::StatusCode::NOT_FOUND, "Session not found").into_response()
    }
}

pub async fn prompt(
    State(_state): State<Arc<AppState>>,
    Path(_session_id): Path<String>,
    Json(_body): Json<PromptBody>,
) -> impl IntoResponse {
    (axum::http::StatusCode::ACCEPTED, "Prompt admission not yet implemented")
}

pub async fn compact(
    State(_state): State<Arc<AppState>>,
    Path(_session_id): Path<String>,
) -> impl IntoResponse {
    (axum::http::StatusCode::OK, "Compaction not yet implemented")
}

pub async fn interrupt(
    State(_state): State<Arc<AppState>>,
    Path(_session_id): Path<String>,
) -> impl IntoResponse {
    (axum::http::StatusCode::OK, "Interrupt not yet implemented")
}

pub async fn wait(
    State(_state): State<Arc<AppState>>,
    Path(_session_id): Path<String>,
) -> impl IntoResponse {
    (axum::http::StatusCode::OK, "Wait not yet implemented")
}

pub async fn revert_stage(
    State(_state): State<Arc<AppState>>,
    Path(_session_id): Path<String>,
    Json(_body): Json<serde_json::Value>,
) -> impl IntoResponse {
    (axum::http::StatusCode::OK, "Revert stage not yet implemented")
}

pub async fn revert_clear(
    State(_state): State<Arc<AppState>>,
    Path(_session_id): Path<String>,
) -> impl IntoResponse {
    (axum::http::StatusCode::OK, "Revert clear not yet implemented")
}

pub async fn revert_commit(
    State(_state): State<Arc<AppState>>,
    Path(_session_id): Path<String>,
) -> impl IntoResponse {
    (axum::http::StatusCode::OK, "Revert commit not yet implemented")
}

pub async fn context(
    State(_state): State<Arc<AppState>>,
    Path(_session_id): Path<String>,
) -> impl IntoResponse {
    Json(serde_json::json!({ "data": [] }))
}

pub async fn history(
    State(_state): State<Arc<AppState>>,
    Path(_session_id): Path<String>,
) -> impl IntoResponse {
    Json(serde_json::json!({ "data": [], "hasMore": false }))
}

pub async fn events(
    State(_state): State<Arc<AppState>>,
    Path(_session_id): Path<String>,
) -> impl IntoResponse {
    (axum::http::StatusCode::OK, "SSE streaming not yet implemented")
}

pub async fn get_message(
    State(_state): State<Arc<AppState>>,
    Path((session_id, message_id)): Path<(String, String)>,
) -> impl IntoResponse {
    (axum::http::StatusCode::NOT_FOUND, format!("Message {} not found in session {}", message_id, session_id))
}

pub async fn list_messages(
    State(_state): State<Arc<AppState>>,
    Path(_session_id): Path<String>,
    Query(_params): Query<serde_json::Value>,
) -> impl IntoResponse {
    Json(serde_json::json!({ "data": [], "cursor": {} }))
}

#[derive(Deserialize)]
pub struct CreateSessionBody {
    id: Option<SessionID>,
    agent: Option<AgentID>,
    model: Option<opencode_schema::model::ModelRef>,
    location: Option<opencode_schema::location::LocationRef>,
}

#[derive(Deserialize)]
pub struct SwitchAgentBody {
    agent: AgentID,
}

#[derive(Deserialize)]
pub struct SwitchModelBody {
    model: opencode_schema::model::ModelRef,
}

#[derive(Deserialize)]
pub struct PromptBody {
    prompt: Prompt,
    delivery: Option<SessionDelivery>,
}


