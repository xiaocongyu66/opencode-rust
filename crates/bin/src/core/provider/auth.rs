//! Provider auth management.
//!
//! Ported from `provider/auth.ts`.
//! Handles OAuth and API key authentication flows for providers.

use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::schema::ids::ProviderID;

/// OAuth auth result.
pub struct AuthOAuthResult {
    pub url: String,
    pub method: String,
    pub instructions: String,
    pub callback: Box<dyn Fn(Option<String>) -> std::pin::Pin<Box<dyn std::future::Future<Output = Option<OAuthCallbackResult>> + Send>> + Send + Sync>,
}

/// OAuth callback result.
#[derive(Debug, Clone)]
pub struct OAuthCallbackResult {
    pub success: bool,
    pub key: Option<String>,
    pub access: Option<String>,
    pub refresh: Option<String>,
    pub expires: Option<u64>,
    pub metadata: Option<HashMap<String, String>>,
}

/// Auth method definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthMethod {
    pub method_type: String,
    pub label: String,
    pub prompts: Option<Vec<AuthPrompt>>,
}

/// Auth prompt.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum AuthPrompt {
    Text {
        key: String,
        message: String,
        placeholder: Option<String>,
        when: Option<When>,
    },
    Select {
        key: String,
        message: String,
        options: Vec<SelectOption>,
        when: Option<When>,
    },
}

/// When condition for prompts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct When {
    pub key: String,
    pub op: String,
    pub value: String,
}

/// Select option.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectOption {
    pub label: String,
    pub value: String,
    pub hint: Option<String>,
}

/// Authorization info.
#[derive(Debug, Clone)]
pub struct Authorization {
    pub url: String,
    pub method: String,
    pub instructions: String,
}

/// Provider auth manager.
pub struct ProviderAuthManager {
    pending: Arc<RwLock<HashMap<ProviderID, AuthOAuthResult>>>,
}

impl ProviderAuthManager {
    pub fn new() -> Self {
        Self {
            pending: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn set_pending(&self, provider_id: ProviderID, result: AuthOAuthResult) {
        self.pending.write().await.insert(provider_id, result);
    }

    pub async fn take_pending(&self, provider_id: &ProviderID) -> Option<AuthOAuthResult> {
        self.pending.write().await.remove(provider_id)
    }
}

impl Default for ProviderAuthManager {
    fn default() -> Self {
        Self::new()
    }
}
