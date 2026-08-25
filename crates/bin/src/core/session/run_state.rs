//! Session run state management.
//!
//! Ported from `session/run-state.ts`.
//! Tracks running sessions, prevents concurrent execution, and manages cancellation.

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::{Mutex, RwLock};
use tokio::task::JoinHandle;

use crate::schema::ids::SessionID;

/// Busy error — raised when a session is already running.
#[derive(Debug, Clone, thiserror::Error)]
#[error("Session is busy: {0}")]
pub struct BusyError(pub SessionID);

/// Session runner state.
struct RunnerState {
    busy: bool,
    cancel_handle: Option<tokio::sync::oneshot::Sender<()>>,
    task: Option<JoinHandle<()>>,
}

impl Default for RunnerState {
    fn default() -> Self {
        Self {
            busy: false,
            cancel_handle: None,
            task: None,
        }
    }
}

/// Session run state manager — tracks active sessions.
pub struct SessionRunState {
    runners: Arc<RwLock<HashMap<SessionID, Mutex<RunnerState>>>>,
}

impl SessionRunState {
    pub fn new() -> Self {
        Self {
            runners: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Assert that a session is not busy.
    pub async fn assert_not_busy(&self, session_id: &SessionID) -> Result<(), BusyError> {
        let runners = self.runners.read().await;
        if let Some(mutex) = runners.get(session_id) {
            let state = mutex.lock().await;
            if state.busy {
                return Err(BusyError(session_id.clone()));
            }
        }
        Ok(())
    }

    /// Cancel a running session.
    pub async fn cancel(&self, session_id: &SessionID) {
        let runners = self.runners.read().await;
        if let Some(mutex) = runners.get(session_id) {
            let mut state = mutex.lock().await;
            if let Some(tx) = state.cancel_handle.take() {
                let _ = tx.send(());
            }
            if let Some(task) = state.task.take() {
                task.abort();
            }
            state.busy = false;
        }
    }

    /// Register a running session.
    pub async fn start(
        &self,
        session_id: SessionID,
    ) -> tokio::sync::oneshot::Receiver<()> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        let runners = self.runners.read().await;
        if let Some(mutex) = runners.get(&session_id) {
            let mut state = mutex.lock().await;
            state.busy = true;
            state.cancel_handle = Some(tx);
        } else {
            drop(runners);
            let mut runners = self.runners.write().await;
            let mut state = runners
                .entry(session_id.clone())
                .or_insert_with(|| Mutex::new(RunnerState::default()))
                .lock()
                .await;
            state.busy = true;
            state.cancel_handle = Some(tx);
        }
        rx
    }

    /// Mark a session as idle (done).
    pub async fn set_idle(&self, session_id: &SessionID) {
        let runners = self.runners.read().await;
        if let Some(mutex) = runners.get(session_id) {
            let mut state = mutex.lock().await;
            state.busy = false;
            state.cancel_handle = None;
            state.task = None;
        }
    }

    /// Check if a session is currently busy.
    pub async fn is_busy(&self, session_id: &SessionID) -> bool {
        let runners = self.runners.read().await;
        if let Some(mutex) = runners.get(session_id) {
            return mutex.lock().await.busy;
        }
        false
    }
}

impl Default for SessionRunState {
    fn default() -> Self {
        Self::new()
    }
}
