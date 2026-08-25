//! Session execution — the agent loop orchestrator.

use opencode_schema::ids::SessionID;
use opencode_schema::session::SessionStatus;

pub struct SessionExecution;

impl SessionExecution {
    pub async fn wake(_session_id: &SessionID) {
        todo!("Session execution wake requires a run coordinator")
    }

    pub async fn status(_session_id: &SessionID) -> SessionStatus {
        todo!("Session status requires a session store")
    }

    pub async fn interrupt(_session_id: &SessionID) {
        todo!("Interrupt requires an active execution context")
    }
}
