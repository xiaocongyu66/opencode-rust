//! Integration management.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::schema::ids::IntegrationID;
use crate::schema::integration::IntegrationInfo;

pub struct IntegrationRegistry {
    integrations: Arc<RwLock<HashMap<String, IntegrationInfo>>>,
}

impl IntegrationRegistry {
    pub fn new() -> Self {
        Self { integrations: Arc::new(RwLock::new(HashMap::new())) }
    }

    pub async fn get(&self, id: &IntegrationID) -> Option<IntegrationInfo> {
        self.integrations.read().await.get(&id.0).cloned()
    }

    pub async fn list(&self) -> Vec<IntegrationInfo> {
        self.integrations.read().await.values().cloned().collect()
    }

    pub async fn register(&self, info: IntegrationInfo) {
        self.integrations.write().await.insert(info.id.0.clone(), info);
    }
}

impl Default for IntegrationRegistry {
    fn default() -> Self {
        Self::new()
    }
}
