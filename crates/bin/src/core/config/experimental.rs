//! Experimental configuration flags.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExperimentalConfig {
    #[serde(flatten)]
    pub flags: HashMap<String, serde_json::Value>,
}
