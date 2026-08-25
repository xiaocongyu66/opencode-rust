//! Event system.

use tokio::sync::broadcast;
use crate::schema::event::EventPayload;

pub struct EventBus {
    sender: broadcast::Sender<EventPayload>,
}

impl EventBus {
    pub fn new(buffer: usize) -> Self {
        let (sender, _) = broadcast::channel(buffer);
        Self { sender }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<EventPayload> {
        self.sender.subscribe()
    }

    pub fn publish(&self, event: EventPayload) {
        let _ = self.sender.send(event);
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new(1024)
    }
}
