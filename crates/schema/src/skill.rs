//! Skill data models.

use serde::{Deserialize, Serialize};

use crate::common::AbsolutePath;

/// Skill source — directory, url, or embedded.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SkillSource {
    #[serde(rename = "directory")]
    Directory { path: AbsolutePath },
    #[serde(rename = "url")]
    Url { url: String },
    #[serde(rename = "embedded")]
    Embedded { skill: Box<SkillInfo> },
}

/// Skill info.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillInfo {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slash: Option<bool>,
    pub location: AbsolutePath,
    pub content: String,
}

impl SkillSource {
    pub fn key(&self) -> String {
        match self {
            SkillSource::Directory { path } => format!("directory:{}", path),
            SkillSource::Url { url } => format!("url:{}", url),
            SkillSource::Embedded { skill } => format!("embedded:{}", skill.name),
        }
    }
}
