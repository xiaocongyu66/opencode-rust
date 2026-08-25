//! Background job management.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;

pub struct BackgroundJobManager {
    jobs: Arc<RwLock<HashMap<String, JoinHandle<()>>>>,
}

impl BackgroundJobManager {
    pub fn new() -> Self {
        Self { jobs: Arc::new(RwLock::new(HashMap::new())) }
    }

    pub async fn spawn(&self, id: String, handle: JoinHandle<()>) {
        self.jobs.write().await.insert(id, handle);
    }

    pub async fn cancel(&self, id: &str) -> bool {
        if let Some(handle) = self.jobs.write().await.remove(id) {
            handle.abort();
            true
        } else {
            false
        }
    }

    pub async fn active_count(&self) -> usize {
        self.jobs.read().await.len()
    }
}

impl Default for BackgroundJobManager {
    fn default() -> Self {
        Self::new()
    }
}
