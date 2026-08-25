//! Route definitions — builds the axum Router from all API groups.

use std::sync::Arc;
use axum::Router;
use crate::core::state::AppState;

pub fn build_router(state: Arc<AppState>) -> Router {
    Router::new()
        .merge(health_routes())
        .merge(session_routes())
        .merge(agent_routes())
        .merge(model_routes())
        .merge(provider_routes())
        .merge(tool_routes())
        .merge(command_routes())
        .merge(event_routes())
        .merge(fs_routes())
        .merge(permission_routes())
        .merge(question_routes())
        .merge(skill_routes())
        .merge(reference_routes())
        .merge(location_routes())
        .merge(integration_routes())
        .merge(credential_routes())
        .merge(pty_routes())
        .merge(project_copy_routes())
        .with_state(state)
}

/// Builds the router with an authorization password applied as global
/// middleware, mirroring `createRoutes(password)` in `handlers/serve.ts`.
pub fn build_router_with_password(state: Arc<AppState>, password: &str) -> Router {
    use axum::middleware::from_fn_with_state;
    use crate::protocol::middleware::authorization::authorization_with_password;

    build_router(state).layer(from_fn_with_state(
        password.to_string(),
        authorization_with_password,
    ))
}

fn health_routes() -> Router<Arc<AppState>> {
    use axum::routing::get;
    Router::new().route("/api/health", get(|| async { axum::Json(serde_json::json!({"status": "ok"})) }))
}

fn session_routes() -> Router<Arc<AppState>> {
    use axum::routing::{get, post};
    Router::new()
        .route("/api/session", get(crate::server::handlers::session::list).post(crate::server::handlers::session::create))
        .route("/api/session/active", get(crate::server::handlers::session::list_active))
        .route("/api/session/{session_id}", get(crate::server::handlers::session::get))
        .route("/api/session/{session_id}/agent", post(crate::server::handlers::session::switch_agent))
        .route("/api/session/{session_id}/model", post(crate::server::handlers::session::switch_model))
        .route("/api/session/{session_id}/prompt", post(crate::server::handlers::session::prompt))
        .route("/api/session/{session_id}/compact", post(crate::server::handlers::session::compact))
        .route("/api/session/{session_id}/wait", post(crate::server::handlers::session::wait))
        .route("/api/session/{session_id}/interrupt", post(crate::server::handlers::session::interrupt))
        .route("/api/session/{session_id}/context", get(crate::server::handlers::session::context))
        .route("/api/session/{session_id}/history", get(crate::server::handlers::session::history))
        .route("/api/session/{session_id}/event", get(crate::server::handlers::session::events))
        .route("/api/session/{session_id}/message", get(crate::server::handlers::session::list_messages))
        .route("/api/session/{session_id}/message/{message_id}", get(crate::server::handlers::session::get_message))
        .route("/api/session/{session_id}/revert/stage", post(crate::server::handlers::session::revert_stage))
        .route("/api/session/{session_id}/revert/clear", post(crate::server::handlers::session::revert_clear))
        .route("/api/session/{session_id}/revert/commit", post(crate::server::handlers::session::revert_commit))
}

fn agent_routes() -> Router<Arc<AppState>> {
    use axum::routing::get;
    Router::new().route("/api/agent", get(crate::server::handlers::agent::list))
}

fn model_routes() -> Router<Arc<AppState>> {
    use axum::routing::get;
    Router::new().route("/api/model", get(crate::server::handlers::model::list))
}

fn provider_routes() -> Router<Arc<AppState>> {
    use axum::routing::get;
    Router::new().route("/api/provider", get(crate::server::handlers::provider::list))
}

fn tool_routes() -> Router<Arc<AppState>> {
    use axum::routing::get;
    Router::new().route("/api/tool", get(crate::server::handlers::tool::list))
}

fn command_routes() -> Router<Arc<AppState>> {
    use axum::routing::get;
    Router::new().route("/api/command", get(crate::server::handlers::command::list))
}

fn event_routes() -> Router<Arc<AppState>> {
    use axum::routing::get;
    Router::new().route("/api/event", get(crate::server::handlers::event::subscribe))
}

fn fs_routes() -> Router<Arc<AppState>> {
    use axum::routing::{get, post};
    Router::new()
        .route("/api/fs/list", post(crate::server::handlers::fs::list))
        .route("/api/fs/find", post(crate::server::handlers::fs::find))
        .route("/api/fs/read/{*path}", get(crate::server::handlers::fs::read))
}

fn permission_routes() -> Router<Arc<AppState>> {
    use axum::routing::{get, post};
    Router::new()
        .route("/api/permission/request", get(crate::server::handlers::permission::list_requests))
        .route("/api/permission/saved", get(crate::server::handlers::permission::list_saved).post(crate::server::handlers::permission::save))
        .route("/api/permission/saved/{id}", get(crate::server::handlers::permission::get_saved).delete(crate::server::handlers::permission::delete_saved))
        .route("/api/session/{session_id}/permission", get(crate::server::handlers::permission::session_list))
        .route("/api/session/{session_id}/permission/{request_id}", get(crate::server::handlers::permission::get_request))
        .route("/api/session/{session_id}/permission/{request_id}/reply", post(crate::server::handlers::permission::reply))
}

fn question_routes() -> Router<Arc<AppState>> {
    use axum::routing::{get, post};
    Router::new()
        .route("/api/question/request", get(crate::server::handlers::question::list_requests))
        .route("/api/session/{session_id}/question", get(crate::server::handlers::question::session_list))
        .route("/api/session/{session_id}/question/{request_id}/reply", post(crate::server::handlers::question::reply))
        .route("/api/session/{session_id}/question/{request_id}/reject", post(crate::server::handlers::question::reject))
}

fn skill_routes() -> Router<Arc<AppState>> {
    use axum::routing::get;
    Router::new().route("/api/skill", get(crate::server::handlers::skill::list))
}

fn reference_routes() -> Router<Arc<AppState>> {
    use axum::routing::get;
    Router::new().route("/api/reference", get(crate::server::handlers::reference::list))
}

fn location_routes() -> Router<Arc<AppState>> {
    use axum::routing::get;
    Router::new().route("/api/location", get(crate::server::handlers::location::list))
}

fn integration_routes() -> Router<Arc<AppState>> {
    use axum::routing::{get, post};
    Router::new()
        .route("/api/integration", get(crate::server::handlers::integration::list))
        .route("/api/integration/{integration_id}", get(crate::server::handlers::integration::get))
        .route("/api/integration/{integration_id}/connect/oauth", post(crate::server::handlers::integration::connect_oauth))
        .route("/api/integration/{integration_id}/connect/key", post(crate::server::handlers::integration::connect_key))
        .route("/api/integration/attempt/{attempt_id}", get(crate::server::handlers::integration::get_attempt))
        .route("/api/integration/attempt/{attempt_id}/complete", post(crate::server::handlers::integration::complete_attempt))
}

fn credential_routes() -> Router<Arc<AppState>> {
    use axum::routing::get;
    Router::new()
        .route("/api/credential/{credential_id}", get(crate::server::handlers::credential::get).delete(crate::server::handlers::credential::delete))
}

fn pty_routes() -> Router<Arc<AppState>> {
    use axum::routing::{get, post};
    Router::new()
        .route("/api/pty", get(crate::server::handlers::pty::list).post(crate::server::handlers::pty::create))
        .route("/api/pty/{pty_id}", get(crate::server::handlers::pty::get))
        .route("/api/pty/{pty_id}/connect", get(crate::server::handlers::pty::connect))
        .route("/api/pty/{pty_id}/connect-token", post(crate::server::handlers::pty::connect_token))
}

fn project_copy_routes() -> Router<Arc<AppState>> {
    use axum::routing::post;
    Router::new()
        .route("/experimental/project/{project_id}/copy", post(crate::server::handlers::project_copy::create).delete(crate::server::handlers::project_copy::delete))
        .route("/experimental/project/{project_id}/copy/refresh", post(crate::server::handlers::project_copy::refresh))
}
