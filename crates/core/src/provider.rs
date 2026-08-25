//! Provider management.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use opencode_schema::ids::ProviderID;
use opencode_schema::provider::ProviderInfo;

pub struct ProviderRegistry {
    providers: Arc<RwLock<HashMap<String, ProviderInfo>>>,
}

impl ProviderRegistry {
    pub fn new() -> Self {
        Self { providers: Arc::new(RwLock::new(HashMap::new())) }
    }

    pub async fn get(&self, id: &ProviderID) -> Option<ProviderInfo> {
        self.providers.read().await.get(id.as_str()).cloned()
    }

    pub async fn list(&self) -> Vec<ProviderInfo> {
        self.providers.read().await.values().cloned().collect()
    }

    pub async fn register(&self, info: ProviderInfo) {
        self.providers.write().await.insert(info.id.0.clone(), info);
    }
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}
