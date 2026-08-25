//! Connection data models.

use serde::{Deserialize, Serialize};

use crate::schema::ids::CredentialID;

/// Credential-based connection info.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub struct ConnectionCredentialInfo {
    pub id: CredentialID,
    pub label: String,
}

/// Environment-based connection info.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub struct ConnectionEnvInfo {
    pub name: String,
}

/// Connection info — either credential or env.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ConnectionInfo {
    #[serde(rename = "credential")]
    Credential {
        #[serde(flatten)]
        inner: ConnectionCredentialInfo,
    },
    #[serde(rename = "env")]
    Env {
        #[serde(flatten)]
        inner: ConnectionEnvInfo,
    },
}
