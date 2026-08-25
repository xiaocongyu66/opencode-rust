//! Model data models.

use serde::{Deserialize, Serialize};

use crate::schema::ids::{ModelID, ModelFamily, ProviderID, VariantID};
use crate::schema::provider::ProviderRequestFields;

/// A reference to a model.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelRef {
    pub id: ModelID,
    pub provider_id: ProviderID,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variant: Option<VariantID>,
}

/// Model capabilities.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelCapabilities {
    pub tools: bool,
    pub input: Vec<String>,
    pub output: Vec<String>,
}

/// Model cost.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelCost {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tier: Option<ModelCostTier>,
    pub input: f64,
    pub output: f64,
    pub cache: ModelCostCache,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelCostTier {
    #[serde(rename = "type")]
    pub tier_type: String,
    pub size: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelCostCache {
    pub read: f64,
    pub write: f64,
}

/// Model API — either AISDK or Native.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ModelApi {
    #[serde(rename = "aisdk")]
    Aisdk {
        id: ModelID,
        package: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        url: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        settings: Option<std::collections::HashMap<String, serde_json::Value>>,
    },
    #[serde(rename = "native")]
    Native {
        id: ModelID,
        #[serde(skip_serializing_if = "Option::is_none")]
        url: Option<String>,
        settings: std::collections::HashMap<String, serde_json::Value>,
    },
}

/// Model variant.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelVariant {
    pub id: VariantID,
    #[serde(flatten)]
    pub request: ProviderRequestFields,
}

/// Model status.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelStatus {
    Alpha,
    Beta,
    Deprecated,
    Active,
}

/// Model limits.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelLimit {
    pub context: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input: Option<i64>,
    pub output: i64,
}

/// Model info.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelInfo {
    pub id: ModelID,
    pub provider_id: ProviderID,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub family: Option<ModelFamily>,
    pub name: String,
    pub api: ModelApi,
    pub capabilities: ModelCapabilities,
    pub request: ModelRequest,
    pub variants: Vec<ModelVariant>,
    pub time: ModelTime,
    pub cost: Vec<ModelCost>,
    pub status: ModelStatus,
    pub enabled: bool,
    pub limit: ModelLimit,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelRequest {
    #[serde(flatten)]
    pub fields: ProviderRequestFields,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variant: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelTime {
    pub released: f64,
}

impl ModelInfo {
    pub fn empty(provider_id: ProviderID, model_id: ModelID) -> Self {
        Self {
            id: model_id.clone(),
            provider_id,
            family: None,
            name: model_id.0.clone(),
            api: ModelApi::Native {
                id: model_id,
                url: None,
                settings: std::collections::HashMap::new(),
            },
            capabilities: ModelCapabilities {
                tools: false,
                input: vec![],
                output: vec![],
            },
            request: ModelRequest::default(),
            variants: vec![],
            time: ModelTime { released: 0.0 },
            cost: vec![],
            status: ModelStatus::Active,
            enabled: true,
            limit: ModelLimit {
                context: 0,
                input: None,
                output: 0,
            },
        }
    }
}
