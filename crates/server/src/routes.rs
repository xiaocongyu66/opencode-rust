//! Route definitions — builds the axum Router from all API groups.

use std::sync::Arc;
use axum::Router;
use opencode_core::state::AppState;

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

fn health_routes() -> Router<Arc<AppState>> {
    use axum::routing::get;
    Router::new().route("/api/health", get(|| async { axum::Json(serde_json::json!({"status": "ok"})) }))
}

fn session_routes() -> Router<Arc<AppState>> {
    use axum::routing::{get, post};
    Router::new()
        .route("/api/session", get(crate::handlers::session::list).post(crate::handlers::session::create))
        .route("/api/session/active", get(crate::handlers::session::list_active))
        .route("/api/session/:session_id", get(crate::handlers::session::get))
        .route("/api/session/:session_id/agent", post(crate::handlers::session::switch_agent))
        .route("/api/session/:session_id/model", post(crate::handlers::session::switch_model))
        .route("/api/session/:session_id/prompt", post(crate::handlers::session::prompt))
        .route("/api/session/:session_id/compact", post(crate::handlers::session::compact))
        .route("/api/session/:session_id/wait", post(crate::handlers::session::wait))
        .route("/api/session/:session_id/interrupt", post(crate::handlers::session::interrupt))
        .route("/api/session/:session_id/context", get(crate::handlers::session::context))
        .route("/api/session/:session_id/history", get(crate::handlers::session::history))
        .route("/api/session/:session_id/event", get(crate::handlers::session::events))
        .route("/api/session/:session_id/message", get(crate::handlers::session::list_messages))
        .route("/api/session/:session_id/message/:message_id", get(crate::handlers::session::get_message))
        .route("/api/session/:session_id/revert/stage", post(crate::handlers::session::revert_stage))
        .route("/api/session/:session_id/revert/clear", post(crate::handlers::session::revert_clear))
        .route("/api/session/:session_id/revert/commit", post(crate::handlers::session::revert_commit))
}

fn agent_routes() -> Router<Arc<AppState>> {
    use axum::routing::get;
    Router::new().route("/api/agent", get(crate::handlers::agent::list))
}

fn model_routes() -> Router<Arc<AppState>> {
    use axum::routing::get;
    Router::new().route("/api/model", get(crate::handlers::model::list))
}

fn provider_routes() -> Router<Arc<AppState>> {
    use axum::routing::get;
    Router::new().route("/api/provider", get(crate::handlers::provider::list))
}

fn tool_routes() -> Router<Arc<AppState>> {
    use axum::routing::get;
    Router::new().route("/api/tool", get(crate::handlers::tool::list))
}

fn command_routes() -> Router<Arc<AppState>> {
    use axum::routing::get;
    Router::new().route("/api/command", get(crate::handlers::command::list))
}

fn event_routes() -> Router<Arc<AppState>> {
    use axum::routing::get;
    Router::new().route("/api/event", get(crate::handlers::event::subscribe))
}

fn fs_routes() -> Router<Arc<AppState>> {
    use axum::routing::{get, post};
    Router::new()
        .route("/api/fs/list", post(crate::handlers::fs::list))
        .route("/api/fs/find", post(crate::handlers::fs::find))
        .route("/api/fs/read/*path", get(crate::handlers::fs::read))
}

fn permission_routes() -> Router<Arc<AppState>> {
    use axum::routing::{get, post};
    Router::new()
        .route("/api/permission/request", get(crate::handlers::permission::list_requests))
        .route("/api/permission/saved", get(crate::handlers::permission::list_saved).post(crate::handlers::permission::save))
        .route("/api/permission/saved/:id", get(crate::handlers::permission::get_saved).delete(crate::handlers::permission::delete_saved))
        .route("/api/session/:session_id/permission", get(crate::handlers::permission::session_list))
        .route("/api/session/:session_id/permission/:request_id", get(crate::handlers::permission::get_request))
        .route("/api/session/:session_id/permission/:request_id/reply", post(crate::handlers::permission::reply))
}

fn question_routes() -> Router<Arc<AppState>> {
    use axum::routing::{get, post};
    Router::new()
        .route("/api/question/request", get(crate::handlers::question::list_requests))
        .route("/api/session/:session_id/question", get(crate::handlers::question::session_list))
        .route("/api/session/:session_id/question/:request_id/reply", post(crate::handlers::question::reply))
        .route("/api/session/:session_id/question/:request_id/reject", post(crate::handlers::question::reject))
}

fn skill_routes() -> Router<Arc<AppState>> {
    use axum::routing::get;
    Router::new().route("/api/skill", get(crate::handlers::skill::list))
}

fn reference_routes() -> Router<Arc<AppState>> {
    use axum::routing::get;
    Router::new().route("/api/reference", get(crate::handlers::reference::list))
}

fn location_routes() -> Router<Arc<AppState>> {
    use axum::routing::get;
    Router::new().route("/api/location", get(crate::handlers::location::list))
}

fn integration_routes() -> Router<Arc<AppState>> {
    use axum::routing::{get, post};
    Router::new()
        .route("/api/integration", get(crate::handlers::integration::list))
        .route("/api/integration/:integration_id", get(crate::handlers::integration::get))
        .route("/api/integration/:integration_id/connect/oauth", post(crate::handlers::integration::connect_oauth))
        .route("/api/integration/:integration_id/connect/key", post(crate::handlers::integration::connect_key))
        .route("/api/integration/attempt/:attempt_id", get(crate::handlers::integration::get_attempt))
        .route("/api/integration/attempt/:attempt_id/complete", post(crate::handlers::integration::complete_attempt))
}

fn credential_routes() -> Router<Arc<AppState>> {
    use axum::routing::get;
    Router::new()
        .route("/api/credential/:credential_id", get(crate::handlers::credential::get).delete(crate::handlers::credential::delete))
}

fn pty_routes() -> Router<Arc<AppState>> {
    use axum::routing::{get, post};
    Router::new()
        .route("/api/pty", get(crate::handlers::pty::list).post(crate::handlers::pty::create))
        .route("/api/pty/:pty_id", get(crate::handlers::pty::get))
        .route("/api/pty/:pty_id/connect", get(crate::handlers::pty::connect))
        .route("/api/pty/:pty_id/connect-token", post(crate::handlers::pty::connect_token))
}

fn project_copy_routes() -> Router<Arc<AppState>> {
    use axum::routing::post;
    Router::new()
        .route("/experimental/project/:project_id/copy", post(crate::handlers::project_copy::create).delete(crate::handlers::project_copy::delete))
        .route("/experimental/project/:project_id/copy/refresh", post(crate::handlers::project_copy::refresh))
}
