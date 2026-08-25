//! Observability — tracing and metrics.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Default)]
pub struct Metrics {
    pub counters: HashMap<String, u64>,
    pub gauges: HashMap<String, f64>,
}

pub struct Observability {
    metrics: Arc<RwLock<Metrics>>,
}

impl Observability {
    pub fn new() -> Self {
        Self { metrics: Arc::new(RwLock::new(Metrics::default())) }
    }

    pub async fn increment(&self, name: &str) {
        let mut m = self.metrics.write().await;
        *m.counters.entry(name.to_string()).or_insert(0) += 1;
    }

    pub async fn set_gauge(&self, name: &str, value: f64) {
        let mut m = self.metrics.write().await;
        m.gauges.insert(name.to_string(), value);
    }

    pub async fn snapshot(&self) -> Metrics {
        self.metrics.read().await.clone()
    }
}

impl Default for Observability {
    fn default() -> Self {
        Self::new()
    }
}
