//! Plugin data models.

use serde::{Deserialize, Serialize};

/// Plugin ID — branded string.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PluginID(pub String);

impl PluginID {
    pub fn from_str(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<String> for PluginID {
    fn from(s: String) -> Self {
        Self(s)
    }
}
