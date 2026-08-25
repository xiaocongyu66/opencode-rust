//! Provider management.
//!
//! Ported from `provider/` directory in the TS original.
//! Re-exports schema provider types and provides registry logic.

pub mod error;
pub mod transform;
pub mod model_status;
pub mod auth;

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::schema::ids::ProviderID;
use crate::schema::provider::ProviderInfo;

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

    pub async fn exists(&self, id: &ProviderID) -> bool {
        self.providers.read().await.contains_key(id.as_str())
    }
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}
