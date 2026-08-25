//! Session schema — ID types for messages and parts.
//!
//! Ported from `session/schema.ts`.

use crate::schema::common::ascending;

/// Session ID — re-exported from schema::ids.
pub use crate::schema::ids::SessionID;

/// Message ID — branded string prefixed with "msg_".
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct MessageID(pub String);

impl MessageID {
    pub fn ascending() -> Self {
        Self(format!("msg_{}", ascending()))
    }

    pub fn from_str(s: &str) -> Self {
        Self(s.to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for MessageID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<String> for MessageID {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for MessageID {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// Part ID — branded string prefixed with "prt_".
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct PartID(pub String);

impl PartID {
    pub fn ascending() -> Self {
        Self(format!("prt_{}", ascending()))
    }

    pub fn from_str(s: &str) -> Self {
        Self(s.to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for PartID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<String> for PartID {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for PartID {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}
