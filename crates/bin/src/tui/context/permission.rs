use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, PartialEq)]
pub enum PermissionMode {
    Auto,
    Normal,
}

pub struct PermissionContext {
    mode: Arc<Mutex<PermissionMode>>,
}

impl PermissionContext {
    pub fn new(auto: bool) -> Self {
        let mode = if auto {
            PermissionMode::Auto
        } else {
            PermissionMode::Normal
        };
        Self {
            mode: Arc::new(Mutex::new(mode)),
        }
    }

    pub fn mode(&self) -> PermissionMode {
        self.mode.lock().unwrap().clone()
    }

    pub fn set(&self, mode: PermissionMode) {
        *self.mode.lock().unwrap() = mode;
    }

    pub fn toggle(&self) {
        let mut guard = self.mode.lock().unwrap();
        *guard = match &*guard {
            PermissionMode::Auto => PermissionMode::Normal,
            PermissionMode::Normal => PermissionMode::Auto,
        };
    }
}
