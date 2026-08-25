//! Strongly-typed identifier types.

use std::fmt;

use crate::schema::common::{ascending, descending};

macro_rules! prefixed_id {
    ($name:ident, $prefix:expr) => {
        #[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
        #[serde(transparent)]
        pub struct $name(pub String);

        impl $name {
            pub fn new() -> Self {
                Self(format!("{}{}", $prefix, ascending()))
            }

            pub fn from_str(s: &str) -> Self {
                Self(s.to_string())
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.0)
            }
        }

        impl From<String> for $name {
            fn from(s: String) -> Self {
                Self(s)
            }
        }

        impl From<&str> for $name {
            fn from(s: &str) -> Self {
                Self(s.to_string())
            }
        }
    };
}

prefixed_id!(SessionID, "ses_");
prefixed_id!(WorkspaceID, "wrk_");
prefixed_id!(PermissionID, "per_");
prefixed_id!(QuestionID, "que_");
prefixed_id!(PtyID, "pty_");
prefixed_id!(SessionMessageID, "msg_");
prefixed_id!(CredentialID, "cred_");
prefixed_id!(IntegrationAttemptID, "con_");

/// Session ID that uses descending (reverse-time) ordering.
pub fn session_id_descending() -> SessionID {
    SessionID(format!("ses_{}", descending()))
}

/// Project ID — plain branded string with a "global" constant.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct ProjectID(pub String);

impl ProjectID {
    pub fn from_str(s: &str) -> Self {
        Self(s.to_string())
    }

    pub fn global() -> Self {
        Self("global".to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ProjectID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<String> for ProjectID {
    fn from(s: String) -> Self {
        Self(s)
    }
}

/// Integration ID — branded string.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct IntegrationID(pub String);

impl fmt::Display for IntegrationID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<String> for IntegrationID {
    fn from(s: String) -> Self {
        Self(s)
    }
}

/// Integration method ID — branded string.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct IntegrationMethodID(pub String);

impl fmt::Display for IntegrationMethodID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<String> for IntegrationMethodID {
    fn from(s: String) -> Self {
        Self(s)
    }
}

/// Event ID — branded string with "evt_" prefix.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct EventID(pub String);

impl EventID {
    pub fn new() -> Self {
        Self(format!("evt_{}", ascending()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for EventID {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for EventID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<String> for EventID {
    fn from(s: String) -> Self {
        Self(s)
    }
}

/// Model ID — branded string.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct ModelID(pub String);

impl fmt::Display for ModelID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<String> for ModelID {
    fn from(s: String) -> Self {
        Self(s)
    }
}

/// Model variant ID — branded string.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct VariantID(pub String);

impl fmt::Display for VariantID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<String> for VariantID {
    fn from(s: String) -> Self {
        Self(s)
    }
}

/// Model family — branded string.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct ModelFamily(pub String);

impl From<String> for ModelFamily {
    fn from(s: String) -> Self {
        Self(s)
    }
}

/// Agent ID — branded string.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct AgentID(pub String);

impl AgentID {
    pub fn from_str(s: &str) -> Self {
        Self(s.to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for AgentID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<String> for AgentID {
    fn from(s: String) -> Self {
        Self(s)
    }
}

/// Provider ID — branded string with well-known constants.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct ProviderID(pub String);

impl ProviderID {
    pub fn opencode() -> Self {
        Self("opencode".to_string())
    }
    pub fn anthropic() -> Self {
        Self("anthropic".to_string())
    }
    pub fn openai() -> Self {
        Self("openai".to_string())
    }
    pub fn google() -> Self {
        Self("google".to_string())
    }
    pub fn google_vertex() -> Self {
        Self("google-vertex".to_string())
    }
    pub fn github_copilot() -> Self {
        Self("github-copilot".to_string())
    }
    pub fn amazon_bedrock() -> Self {
        Self("amazon-bedrock".to_string())
    }
    pub fn azure() -> Self {
        Self("azure".to_string())
    }
    pub fn openrouter() -> Self {
        Self("openrouter".to_string())
    }
    pub fn mistral() -> Self {
        Self("mistral".to_string())
    }
    pub fn gitlab() -> Self {
        Self("gitlab".to_string())
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ProviderID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<String> for ProviderID {
    fn from(s: String) -> Self {
        Self(s)
    }
}

/// Plugin ID — branded string.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct PluginID(pub String);

impl fmt::Display for PluginID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<String> for PluginID {
    fn from(s: String) -> Self {
        Self(s)
    }
}
