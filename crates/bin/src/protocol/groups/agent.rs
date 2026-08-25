//! Agent routes — `packages/protocol/src/groups/agent.ts`

use axum::Router;

use crate::protocol::api::ApiGroup;

/// `GET` — List currently registered agents.
pub const AGENT_LIST: &str = "/api/agent";

/// Agent API group.
pub struct AgentGroup;

impl ApiGroup for AgentGroup {
    type State = ();
    fn routes() -> Router<()> {
        Router::new()
    }
}
