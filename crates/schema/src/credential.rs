//! Credential data models.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::ids::IntegrationMethodID;

/// OAuth credential value.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub struct CredentialOAuth {
    pub method_id: IntegrationMethodID,
    pub refresh: String,
    pub access: String,
    pub expires: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// API key credential value.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub struct CredentialKey {
    pub key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Credential value — either OAuth or Key.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CredentialValue {
    #[serde(rename = "oauth")]
    Oauth {
        #[serde(flatten)]
        inner: CredentialOAuth,
    },
    #[serde(rename = "key")]
    Key {
        #[serde(flatten)]
        inner: CredentialKey,
    },
}
