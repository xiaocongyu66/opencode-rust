//! Session handlers.

use std::sync::Arc;
use axum::extract::{Path, Query, State};
use axum::response::IntoResponse;
use axum::Json;
use crate::core::state::AppState;
use crate::schema::ids::{AgentID, SessionID};
use crate::schema::prompt::Prompt;
use crate::schema::session::{SessionDelivery, SessionInfo};
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
        project_id: crate::schema::ids::ProjectID::global(),
        agent: body.agent,
        model: body.model,
        cost: 0.0,
        tokens: Default::default(),
        time: crate::schema::session::SessionTime {
            created: now,
            updated: now,
            archived: None,
        },
        title: crate::core::session::default_parent_title(),
        location: body.location.unwrap_or(crate::schema::location::LocationRef {
            directory: crate::schema::common::AbsolutePath::new(
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
    Path(session_id): Path<String>,
    Json(body): Json<PromptBody>,
) -> impl IntoResponse {
    let now = chrono::Utc::now();
    let delivery = body.delivery.unwrap_or(SessionDelivery::Steer);
    Json(serde_json::json!({
        "data": {
            "admittedSeq": 0,
            "id": "msg_0",
            "sessionID": session_id,
            "prompt": serde_json::to_value(&body.prompt).unwrap_or(serde_json::Value::Null),
            "delivery": serde_json::to_value(&delivery).unwrap_or(serde_json::json!("steer")),
            "timeCreated": now,
        }
    }))
}

pub async fn compact(
    State(_state): State<Arc<AppState>>,
    Path(_session_id): Path<String>,
) -> impl IntoResponse {
    Json(serde_json::json!({ "data": null }))
}

pub async fn interrupt(
    State(_state): State<Arc<AppState>>,
    Path(_session_id): Path<String>,
) -> impl IntoResponse {
    Json(serde_json::json!({ "data": null }))
}

pub async fn wait(
    State(_state): State<Arc<AppState>>,
    Path(_session_id): Path<String>,
) -> impl IntoResponse {
    Json(serde_json::json!({ "data": null }))
}

pub async fn revert_stage(
    State(_state): State<Arc<AppState>>,
    Path(_session_id): Path<String>,
    Json(_body): Json<serde_json::Value>,
) -> impl IntoResponse {
    Json(serde_json::json!({ "data": {} }))
}

pub async fn revert_clear(
    State(_state): State<Arc<AppState>>,
    Path(_session_id): Path<String>,
) -> impl IntoResponse {
    Json(serde_json::json!({ "data": {} }))
}

pub async fn revert_commit(
    State(_state): State<Arc<AppState>>,
    Path(_session_id): Path<String>,
) -> impl IntoResponse {
    Json(serde_json::json!({ "data": {} }))
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
    axum::response::Response::builder()
        .header("content-type", "text/event-stream")
        .header("cache-control", "no-cache")
        .body(axum::body::Body::empty())
        .unwrap()
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
    model: Option<crate::schema::model::ModelRef>,
    location: Option<crate::schema::location::LocationRef>,
}

#[derive(Deserialize)]
pub struct SwitchAgentBody {
    agent: AgentID,
}

#[derive(Deserialize)]
pub struct SwitchModelBody {
    model: crate::schema::model::ModelRef,
}

#[derive(Deserialize)]
pub struct PromptBody {
    prompt: Prompt,
    delivery: Option<SessionDelivery>,
}


