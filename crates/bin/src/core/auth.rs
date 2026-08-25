//! Auth management.
//!
//! Ported from `auth/index.ts`.
//! Handles API keys and OAuth tokens for providers.

use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

pub const OAUTH_DUMMY_KEY: &str = "opencode-oauth-dummy-key";

/// OAuth auth info.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub struct OAuthInfo {
    pub refresh: String,
    pub access: String,
    pub expires: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enterprise_url: Option<String>,
}

/// API key auth info.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiAuth {
    pub key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
}

/// Well-known auth info.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WellKnownAuth {
    pub key: String,
    pub token: String,
}

/// Auth info — discriminated union.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum AuthInfo {
    Oauth(OAuthInfo),
    Api(ApiAuth),
    Wellknown(WellKnownAuth),
}

/// Auth error.
#[derive(Debug, Clone, thiserror::Error)]
pub enum AuthError {
    #[error("{0}")]
    Message(String),
    #[error("Failed to read auth data")]
    ReadError,
    #[error("Failed to write auth data")]
    WriteError,
}

/// Auth manager — stores and retrieves auth info per provider.
pub struct AuthManager {
    data: Arc<RwLock<HashMap<String, AuthInfo>>>,
    file_path: std::path::PathBuf,
}

impl AuthManager {
    pub fn new(data_dir: &str) -> Self {
        let file_path = std::path::Path::new(data_dir).join("auth.json");
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
            file_path,
        }
    }

    /// Load auth data from file.
    pub async fn load(&self) -> Result<(), AuthError> {
        if let Ok(content) = tokio::fs::read_to_string(&self.file_path).await {
            if let Ok(data) = serde_json::from_str::<HashMap<String, AuthInfo>>(&content) {
                *self.data.write().await = data;
            }
        }
        Ok(())
    }

    /// Save auth data to file.
    pub async fn save(&self) -> Result<(), AuthError> {
        let data = self.data.read().await;
        let json = serde_json::to_string_pretty(&*data).map_err(|_| AuthError::WriteError)?;
        tokio::fs::write(&self.file_path, json)
            .await
            .map_err(|_| AuthError::WriteError)?;
        Ok(())
    }

    /// Get auth info for a provider.
    pub async fn get(&self, provider_id: &str) -> Option<AuthInfo> {
        self.data.read().await.get(provider_id).cloned()
    }

    /// Get all auth info.
    pub async fn all(&self) -> HashMap<String, AuthInfo> {
        self.data.read().await.clone()
    }

    /// Set auth info for a provider.
    pub async fn set(&self, key: &str, info: AuthInfo) -> Result<(), AuthError> {
        let norm = key.trim_end_matches('/');
        let mut data = self.data.write().await;
        data.insert(norm.to_string(), info);
        drop(data);
        self.save().await
    }

    /// Remove auth info for a provider.
    pub async fn remove(&self, key: &str) -> Result<(), AuthError> {
        let norm = key.trim_end_matches('/');
        let mut data = self.data.write().await;
        data.remove(key);
        data.remove(norm);
        drop(data);
        self.save().await
    }
}
