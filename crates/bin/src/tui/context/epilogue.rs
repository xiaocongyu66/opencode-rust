use std::sync::{Arc, Mutex};

pub struct EpilogueContext {
    setter: Arc<Mutex<Option<Box<dyn FnOnce(Option<String>) + Send + Sync>>>>,
}

impl EpilogueContext {
    pub fn new<F>(f: F) -> Self
    where
        F: FnOnce(Option<String>) + Send + Sync + 'static,
    {
        Self {
            setter: Arc::new(Mutex::new(Some(Box::new(f)))),
        }
    }

    pub fn set(&self, value: Option<String>) {
        if let Some(f) = self.setter.lock().unwrap().take() {
            f(value);
        }
    }
}
