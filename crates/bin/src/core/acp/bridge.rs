//! ACP bridge — converts RunnerEvent stream to AcpEvent stream.
//!
//! Sits between the runner (which emits RunnerEvent via mpsc) and the
//! frontend (which subscribes to AcpEvent). This decouples frontends
//! from the runner implementation — swap the agent core without touching
//! TUI/print/IDE code.
//!
//! claude-code-book Ch13: the bridge is the "event envelope" layer.

use tokio::sync::mpsc;

use crate::core::session::runner::RunnerEvent;

use super::protocol::{from_runner_event, AcpEvent};

/// Spawn a background task that drains `runner_rx` and forwards converted
/// AcpEvents to `acp_tx`. Returns immediately; the task runs until the
/// runner channel closes or all ACP subscribers drop.
pub fn spawn_bridge(
    mut runner_rx: mpsc::Receiver<RunnerEvent>,
    acp_tx: mpsc::Sender<AcpEvent>,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut step = 0usize;
        while let Some(event) = runner_rx.recv().await {
            // Track current step for request_id when the event doesn't carry it.
            if let RunnerEvent::StepStarted { step: s } = &event {
                step = *s;
            }
            if let Some(acp) = from_runner_event(event, step) {
                // If send fails, all subscribers dropped — stop the bridge.
                if acp_tx.send(acp).await.is_err() {
                    break;
                }
            }
        }
    })
}
