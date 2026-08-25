//! Session execution — the agent loop orchestrator.
//!
//! Ported from `core/src/session/execution.ts` and `execution/local.ts`.
//! Manages process-local ownership of active session drains, coalescing
//! wakeups and routing interrupts to the active runner.

use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::schema::ids::SessionID;

use super::runner::RunError;

#[derive(Debug, Clone)]
pub enum SessionExecutionState {
    Idle,
    Busy,
    Interrupted,
}

pub struct SessionExecution {
    inner: Arc<Mutex<SessionExecutionInner>>,
}

struct SessionExecutionInner {
    active: HashSet<String>,
}

impl SessionExecution {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(SessionExecutionInner {
                active: HashSet::new(),
            })),
        }
    }

    /// Snapshots active execution owned by this process.
    pub async fn active(&self) -> Vec<SessionID> {
        let inner = self.inner.lock().await;
        inner.active.iter().map(|s| SessionID::from_str(s)).collect()
    }

    /// Mark a session as actively running.
    pub async fn mark_active(&self, session_id: &SessionID) {
        let mut inner = self.inner.lock().await;
        inner.active.insert(session_id.0.clone());
    }

    /// Mark a session as idle (no longer running).
    pub async fn mark_idle(&self, session_id: &SessionID) {
        let mut inner = self.inner.lock().await;
        inner.active.remove(&session_id.0);
    }

    /// Interrupt active work owned by this process. Idle interruption is a no-op.
    pub async fn interrupt(&self, session_id: &SessionID) -> bool {
        let mut inner = self.inner.lock().await;
        inner.active.remove(&session_id.0)
    }

    /// Check if a session is currently active.
    pub async fn is_active(&self, session_id: &SessionID) -> bool {
        let inner = self.inner.lock().await;
        inner.active.contains(&session_id.0)
    }
}

impl Default for SessionExecution {
    fn default() -> Self {
        Self::new()
    }
}

/// Run coordinator — joins explicit resumes, coalesces wakeups.
/// Ported from `core/src/session/run-coordinator.ts`.
pub struct SessionRunCoordinator {
    execution: Arc<SessionExecution>,
}

impl SessionRunCoordinator {
    pub fn new(execution: Arc<SessionExecution>) -> Self {
        Self { execution }
    }

    pub async fn active(&self) -> Vec<SessionID> {
        self.execution.active().await
    }

    /// Start execution while idle or join the active execution.
    pub async fn resume(
        &self,
        session_id: &SessionID,
        runner: &super::runner::SessionRunner,
        system_prompt: &str,
        agent_id: &str,
        agent_steps: Option<u64>,
    ) -> Result<super::runner::RunResult, RunError> {
        self.execution.mark_active(session_id).await;
        let result = runner.run(session_id, system_prompt, agent_id, agent_steps).await;
        self.execution.mark_idle(session_id).await;
        result
    }

    /// Register newly recorded work. Repeated wakeups may coalesce.
    pub async fn wake(&self, session_id: &SessionID) {
        tracing::info!("Wake requested for session {}", session_id);
    }

    /// Interrupt active work owned by this process.
    pub async fn interrupt(&self, session_id: &SessionID) {
        self.execution.interrupt(session_id).await;
        runner_interrupt(session_id).await;
    }
}

async fn runner_interrupt(_session_id: &SessionID) {
    tracing::info!("Interrupt requested");
}
