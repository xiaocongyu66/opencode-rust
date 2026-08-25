use super::event::{EventContext, GlobalEvent};
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub struct SdkContext {
    pub url: String,
    pub directory: Option<String>,
    pub event: Arc<EventContext>,
    abort: Arc<Mutex<bool>>,
    handlers: Arc<Mutex<HashSet<usize>>>,
}

impl SdkContext {
    pub fn new(url: String, directory: Option<String>) -> Self {
        Self {
            url,
            directory,
            event: Arc::new(EventContext::new()),
            abort: Arc::new(Mutex::new(false)),
            handlers: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    pub fn is_aborted(&self) -> bool {
        *self.abort.lock().unwrap()
    }

    pub fn abort(&self) {
        *self.abort.lock().unwrap() = true;
    }

    pub fn handle_event(&self, event: GlobalEvent) {
        self.event.emit(&event);
    }

    pub fn retry_delay(attempt: u32) -> Duration {
        let base = 1000u64;
        let max = 30000u64;
        Duration::from_millis(std::cmp::min(base * 2u64.pow(attempt.saturating_sub(1)), max))
    }
}

pub struct EventBatcher {
    queue: Arc<Mutex<Vec<GlobalEvent>>>,
    last_flush: Arc<Mutex<Instant>>,
}

impl EventBatcher {
    pub fn new() -> Self {
        Self {
            queue: Arc::new(Mutex::new(Vec::new())),
            last_flush: Arc::new(Mutex::new(Instant::now())),
        }
    }

    pub fn push(&self, event: GlobalEvent) -> (Vec<GlobalEvent>, bool) {
        let mut queue = self.queue.lock().unwrap();
        queue.push(event);
        let elapsed = self.last_flush.lock().unwrap().elapsed();
        if elapsed < Duration::from_millis(16) {
            (Vec::new(), false)
        } else {
            let events = std::mem::take(&mut *queue);
            *self.last_flush.lock().unwrap() = Instant::now();
            (events, true)
        }
    }

    pub fn flush(&self) -> Vec<GlobalEvent> {
        let mut queue = self.queue.lock().unwrap();
        let events = std::mem::take(&mut *queue);
        *self.last_flush.lock().unwrap() = Instant::now();
        events
    }
}

impl Default for EventBatcher {
    fn default() -> Self {
        Self::new()
    }
}
