//! Integration data models.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::schema::common::ascending;
use crate::schema::connection::ConnectionInfo;
use crate::schema::ids::{IntegrationID, IntegrationMethodID};

/// Integration "when" condition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationWhen {
    pub key: String,
    pub op: IntegrationWhenOp,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IntegrationWhenOp {
    Eq,
    Neq,
}

/// Integration prompt types.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum IntegrationPrompt {
    #[serde(rename = "text")]
    Text {
        key: String,
        message: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        placeholder: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        when: Option<IntegrationWhen>,
    },
    #[serde(rename = "select")]
    Select {
        key: String,
        message: String,
        options: Vec<IntegrationSelectOption>,
        #[serde(skip_serializing_if = "Option::is_none")]
        when: Option<IntegrationWhen>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationSelectOption {
    pub label: String,
    pub value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
}

/// Integration method — OAuth, Key, or Env.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum IntegrationMethod {
    #[serde(rename = "oauth")]
    Oauth {
        id: IntegrationMethodID,
        label: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        prompts: Option<Vec<IntegrationPrompt>>,
    },
    #[serde(rename = "key")]
    Key {
        #[serde(skip_serializing_if = "Option::is_none")]
        label: Option<String>,
    },
    #[serde(rename = "env")]
    Env {
        names: Vec<String>,
    },
}

/// Integration inputs.
pub type IntegrationInputs = HashMap<String, String>;

/// Integration reference.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationRef {
    pub id: IntegrationID,
    pub name: String,
}

/// Integration info.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationInfo {
    pub id: IntegrationID,
    pub name: String,
    pub methods: Vec<IntegrationMethod>,
    pub connections: Vec<ConnectionInfo>,
}

/// Integration attempt time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationAttemptTime {
    pub created: f64,
    pub expires: f64,
}

/// Integration attempt ID.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AttemptID(pub String);

impl AttemptID {
    pub fn new() -> Self {
        Self(format!("con_{}", ascending()))
    }
}

impl Default for AttemptID {
    fn default() -> Self {
        Self::new()
    }
}

/// Integration attempt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationAttempt {
    pub attempt_id: AttemptID,
    pub url: String,
    pub instructions: String,
    pub mode: IntegrationAttemptMode,
    pub time: IntegrationAttemptTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IntegrationAttemptMode {
    Auto,
    Code,
}

/// Integration attempt status.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum IntegrationAttemptStatus {
    Pending { time: IntegrationAttemptTime },
    Complete { time: IntegrationAttemptTime },
    Failed { message: String, time: IntegrationAttemptTime },
    Expired { time: IntegrationAttemptTime },
}
