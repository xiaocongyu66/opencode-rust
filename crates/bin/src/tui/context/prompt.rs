use std::sync::{Arc, Mutex};

#[derive(Default)]
pub struct PromptRefContext {
    current: Arc<Mutex<Option<String>>>,
}

impl PromptRefContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn current(&self) -> Option<String> {
        self.current.lock().unwrap().clone()
    }

    pub fn set(&self, ref_id: Option<String>) {
        *self.current.lock().unwrap() = ref_id;
    }
}
