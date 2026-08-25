//! Markdown rendering configuration.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarkdownConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub theme: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code_highlight: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_wrap: Option<bool>,
}
