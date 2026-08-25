//! Command configuration.

use serde::{Deserialize, Serialize};
use crate::schema::model::ModelRef;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CommandConfig {
    pub name: String,
    pub template: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<ModelRef>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtask: Option<bool>,
}
