//! Provider configuration.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Model cost configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderModelCostCache {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub read: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub write: Option<f64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderModelCost {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tier: Option<ProviderModelCostTier>,
    pub input: f64,
    pub output: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache: Option<ProviderModelCostCache>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderModelCostTier {
    #[serde(rename = "type")]
    pub tier_type: String,
    pub size: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderModelLimit {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<i64>,
}

/// Model API configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ProviderModelApi {
    #[serde(rename = "aisdk")]
    Aisdk {
        #[serde(skip_serializing_if = "Option::is_none")]
        id: Option<String>,
        package: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        url: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        settings: Option<HashMap<String, serde_json::Value>>,
    },
    #[serde(rename = "native")]
    Native {
        #[serde(skip_serializing_if = "Option::is_none")]
        id: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        url: Option<String>,
        settings: HashMap<String, serde_json::Value>,
    },
}

/// Model configuration within a provider.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderModelConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api: Option<ProviderModelApi>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cost: Option<Vec<ProviderModelCost>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<ProviderModelLimit>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variants: Option<HashMap<String, ProviderModelVariant>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderModelVariant {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<serde_json::Map<String, serde_json::Value>>,
}

/// Provider request configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderRequestConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<serde_json::Map<String, serde_json::Value>>,
}

/// Provider configuration entry.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settings: Option<HashMap<String, serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api: Option<ProviderModelApi>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request: Option<ProviderRequestConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub models: Option<HashMap<String, ProviderModelConfig>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub npm: Option<String>,
}
