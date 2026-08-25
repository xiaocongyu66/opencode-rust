use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct EventMetadata {
    pub directory: String,
    pub workspace: Option<String>,
}

#[derive(Debug, Clone)]
pub struct GlobalEvent {
    pub event_type: String,
    pub payload: serde_json::Value,
    pub directory: String,
    pub workspace: Option<String>,
}

type EventHandler = Box<dyn Fn(&GlobalEvent) + Send + Sync>;

pub struct EventContext {
    handlers: Arc<Mutex<Vec<Arc<EventHandler>>>>,
}

impl EventContext {
    pub fn new() -> Self {
        Self {
            handlers: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn subscribe<F>(&self, handler: F) -> impl FnOnce() + Send + Sync + 'static
    where
        F: Fn(&GlobalEvent) + Send + Sync + 'static,
    {
        let handler: Arc<EventHandler> = Arc::new(Box::new(handler));
        self.handlers.lock().unwrap().push(handler.clone());
        let handlers = self.handlers.clone();
        move || {
            let mut guard = handlers.lock().unwrap();
            guard.retain(|h| !Arc::ptr_eq(h, &handler));
        }
    }

    pub fn on<F>(&self, event_type: &str, handler: F) -> impl FnOnce() + Send + Sync + 'static
    where
        F: Fn(&serde_json::Value, &EventMetadata) + Send + Sync + 'static,
    {
        let et = event_type.to_string();
        self.subscribe(move |event| {
            if event.event_type != et {
                return;
            }
            let metadata = EventMetadata {
                directory: event.directory.clone(),
                workspace: event.workspace.clone(),
            };
            handler(&event.payload, &metadata);
        })
    }

    pub fn emit(&self, event: &GlobalEvent) {
        let handlers = self.handlers.lock().unwrap().clone();
        for handler in &handlers {
            handler(event);
        }
    }
}

impl Default for EventContext {
    fn default() -> Self {
        Self::new()
    }
}
