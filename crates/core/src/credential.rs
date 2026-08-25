//! Credential management.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use opencode_schema::ids::CredentialID;
use opencode_schema::credential::CredentialValue;

pub struct CredentialStore {
    credentials: Arc<RwLock<HashMap<String, CredentialValue>>>,
}

impl CredentialStore {
    pub fn new() -> Self {
        Self { credentials: Arc::new(RwLock::new(HashMap::new())) }
    }

    pub async fn get(&self, id: &CredentialID) -> Option<CredentialValue> {
        self.credentials.read().await.get(&id.0).cloned()
    }

    pub async fn store(&self, id: CredentialID, value: CredentialValue) {
        self.credentials.write().await.insert(id.0, value);
    }

    pub async fn delete(&self, id: &CredentialID) -> bool {
        self.credentials.write().await.remove(&id.0).is_some()
    }
}

impl Default for CredentialStore {
    fn default() -> Self {
        Self::new()
    }
}
