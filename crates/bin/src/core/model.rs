//! Model management.
//!
//! Ported from `core/src/model.ts`.
//! Re-exports schema model types and provides parse/registry logic.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::schema::ids::{ModelID, ProviderID};
use crate::schema::model::{ModelInfo, ModelRef};

pub struct ModelRegistry {
    models: Arc<RwLock<HashMap<String, ModelInfo>>>,
}

impl ModelRegistry {
    pub fn new() -> Self {
        Self { models: Arc::new(RwLock::new(HashMap::new())) }
    }

    pub async fn get(&self, provider: &ProviderID, model: &ModelID) -> Option<ModelInfo> {
        let key = format!("{}:{}", provider, model);
        self.models.read().await.get(&key).cloned()
    }

    pub async fn list(&self, provider: Option<&ProviderID>) -> Vec<ModelInfo> {
        let models = self.models.read().await;
        models
            .values()
            .filter(|m| provider.is_none_or(|p| &m.provider_id == p))
            .cloned()
            .collect()
    }

    pub async fn register(&self, info: ModelInfo) {
        let key = format!("{}:{}", info.provider_id, info.id);
        self.models.write().await.insert(key, info);
    }

    pub async fn resolve(&self, r#ref: &ModelRef) -> Option<ModelInfo> {
        self.get(&r#ref.provider_id, &r#ref.id).await
    }

    /// Get the default model for a provider.
    pub async fn default_model(&self, provider: &ProviderID) -> Option<ModelInfo> {
        let models = self.models.read().await;
        models
            .values()
            .find(|m| &m.provider_id == provider && m.enabled)
            .cloned()
    }

    /// List all enabled models.
    pub async fn enabled(&self) -> Vec<ModelInfo> {
        let models = self.models.read().await;
        models.values().filter(|m| m.enabled).cloned().collect()
    }
}

impl Default for ModelRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Parse a "provider/model" string into provider and model IDs.
pub fn parse(input: &str) -> (ProviderID, ModelID) {
    let (provider, model) = input.split_once('/').unwrap_or((input, ""));
    (
        ProviderID(provider.to_string()),
        ModelID(model.to_string()),
    )
}
