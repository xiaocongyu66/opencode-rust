use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Default)]
pub struct LocationRef {
    pub directory: String,
    pub workspace_id: Option<String>,
}

pub struct LocationContext {
    current: Arc<Mutex<Option<LocationRef>>>,
}

impl LocationContext {
    pub fn new() -> Self {
        Self {
            current: Arc::new(Mutex::new(None)),
        }
    }

    pub fn with(location: Option<LocationRef>) -> Self {
        Self {
            current: Arc::new(Mutex::new(location)),
        }
    }

    pub fn get(&self) -> Option<LocationRef> {
        self.current.lock().unwrap().clone()
    }

    pub fn set(&self, location: LocationRef) {
        *self.current.lock().unwrap() = Some(location);
    }
}

impl Default for LocationContext {
    fn default() -> Self {
        Self::new()
    }
}
