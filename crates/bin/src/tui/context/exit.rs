pub type Exit = Box<dyn FnOnce(Option<String>) + Send + Sync + 'static>;

pub struct ExitContext {
    exit: Arc<Mutex<Option<Exit>>>,
}

impl ExitContext {
    pub fn new(exit: Exit) -> Self {
        Self {
            exit: Arc::new(Mutex::new(Some(exit))),
        }
    }

    pub fn exit(&self, reason: Option<String>) {
        if let Some(f) = self.exit.lock().unwrap().take() {
            f(reason);
        }
    }
}
