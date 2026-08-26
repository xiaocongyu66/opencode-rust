//! Background agent registry (claude-code-book Ch09).
//!
//! Background sub-agents survive parent turn boundaries. When the main
//! turn ends (TurnSuspended / TurnInterrupted), the parent's state is
//! reset, but bg agents keep running. Their streaming chunks must NOT
//! be routed to the main reply bubble after the turn boundary clears
//! their accumulator — they belong to the bg agent's own session.
//!
//! This module tracks which agent IDs are currently running in the
//! background so the event router can skip their chunks.

use std::collections::HashSet;
use std::sync::Arc;

use parking_lot::RwLock;

/// Global set of currently-running background agent IDs.
/// Agents register here when spawned in background mode; unregister on exit.
static BG_AGENT_IDS: std::sync::OnceLock<Arc<RwLock<HashSet<String>>>> =
    std::sync::OnceLock::new();

fn registry() -> Arc<RwLock<HashSet<String>>> {
    BG_AGENT_IDS
        .get_or_init(|| Arc::new(RwLock::new(HashSet::new())))
        .clone()
}

/// Register a background agent. Called when a sub-agent spawns with
/// `run_in_background: true`.
pub fn register(agent_id: &str) {
    let mut set = registry().write();
    set.insert(agent_id.to_string());
    tracing::debug!(agent_id, "bg agent registered");
}

/// Unregister a background agent. Called when the agent finishes or errors.
pub fn unregister(agent_id: &str) {
    let mut set = registry().write();
    set.remove(agent_id);
    tracing::debug!(agent_id, "bg agent unregistered");
}

/// Check if an agent is currently running in the background.
/// Used by the event router to decide where to send the agent's chunks.
pub fn is_background(agent_id: &str) -> bool {
    let set = registry().read();
    set.contains(agent_id)
}

/// Snapshot of all currently-registered bg agent IDs.
pub fn all() -> Vec<String> {
    let set = registry().read();
    set.iter().cloned().collect()
}

/// Clear all bg agents (e.g. on session reset / full interrupt).
pub fn clear() {
    let mut set = registry().write();
    set.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_and_check() {
        // Use a unique ID to avoid test interference.
        let id = format!("test-bg-{}", std::process::id());
        register(&id);
        assert!(is_background(&id));
        unregister(&id);
        assert!(!is_background(&id));
    }

    #[test]
    fn test_clear() {
        let id = format!("test-clear-{}", std::process::id());
        register(&id);
        clear();
        assert!(!is_background(&id));
    }

    #[test]
    fn test_all_returns_snapshot() {
        let id = format!("test-snap-{}", std::process::id());
        register(&id);
        let snapshot = all();
        assert!(snapshot.contains(&id));
        unregister(&id);
    }
}
