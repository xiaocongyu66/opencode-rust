//! Provider data models.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::ids::ProviderID;
use crate::ids::IntegrationID;

/// AISDK-based provider API.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub struct ProviderAISDK {
    pub package: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settings: Option<HashMap<String, serde_json::Value>>,
}

/// Native provider API.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub struct ProviderNative {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    pub settings: HashMap<String, serde_json::Value>,
}

/// Provider API type — either AISDK or Native.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ProviderApi {
    #[serde(rename = "aisdk")]
    Aisdk {
        #[serde(flatten)]
        inner: ProviderAISDK,
    },
    #[serde(rename = "native")]
    Native {
        #[serde(flatten)]
        inner: ProviderNative,
    },
}

/// Provider request shape (headers + body).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProviderRequestFields {
    pub headers: HashMap<String, String>,
    pub body: serde_json::Map<String, serde_json::Value>,
}

/// Provider info.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderInfo {
    pub id: ProviderID,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub integration_id: Option<IntegrationID>,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled: Option<bool>,
    pub api: ProviderApi,
    pub request: ProviderRequestFields,
}

impl ProviderInfo {
    pub fn empty(id: ProviderID) -> Self {
        let name = id.0.clone();
        Self {
            id,
            integration_id: None,
            name,
            disabled: None,
            api: ProviderApi::Native {
                inner: ProviderNative {
                    url: None,
                    settings: HashMap::new(),
                },
            },
            request: ProviderRequestFields::default(),
        }
    }
}
