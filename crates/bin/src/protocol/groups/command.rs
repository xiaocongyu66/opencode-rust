//! Command routes — `packages/protocol/src/groups/command.ts`

use axum::Router;

use crate::protocol::api::ApiGroup;

/// `GET` — List currently registered commands.
pub const COMMAND_LIST: &str = "/api/command";

/// Command API group.
pub struct CommandGroup;

impl ApiGroup for CommandGroup {
    type State = ();
    fn routes() -> Router<()> {
        Router::new()
    }
}
