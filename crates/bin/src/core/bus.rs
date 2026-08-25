//! Global event bus.
//!
//! Ported from `bus/global.ts`.
//! A process-global event emitter for cross-module communication.

use std::sync::Arc;

use tokio::sync::broadcast;

use crate::schema::event::EventPayload;
use crate::schema::ids::EventID;

/// A global event with optional directory/project/workspace context.
#[derive(Debug, Clone)]
pub struct GlobalEvent {
    pub directory: Option<String>,
    pub project: Option<String>,
    pub workspace: Option<String>,
    pub payload: EventPayload,
}

/// Global bus emitter — wraps a broadcast channel.
pub struct GlobalBus {
    sender: broadcast::Sender<GlobalEvent>,
}

impl GlobalBus {
    pub fn new(buffer: usize) -> Self {
        let (sender, _) = broadcast::channel(buffer);
        Self { sender }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<GlobalEvent> {
        self.sender.subscribe()
    }

    pub fn emit(&self, mut event: GlobalEvent) {
        if event.payload.event_type.is_empty() {
            event.payload.id = EventID::new();
        }
        let _ = self.sender.send(event);
    }
}

impl Default for GlobalBus {
    fn default() -> Self {
        Self::new(1024)
    }
}

/// Process-global bus instance.
pub fn global() -> Arc<GlobalBus> {
    use std::sync::OnceLock;
    static BUS: OnceLock<Arc<GlobalBus>> = OnceLock::new();
    BUS.get_or_init(|| Arc::new(GlobalBus::default()))
        .clone()
}
