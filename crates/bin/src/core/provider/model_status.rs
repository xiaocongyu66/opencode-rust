//! Provider model status.
//!
//! Ported from `provider/model-status.ts`.

use serde::{Deserialize, Serialize};

/// Model lifecycle status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ModelStatus {
    Alpha,
    Beta,
    Deprecated,
    Active,
}

impl Default for ModelStatus {
    fn default() -> Self {
        Self::Active
    }
}
