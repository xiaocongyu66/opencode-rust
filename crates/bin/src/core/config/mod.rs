//! Configuration management.
//!
//! Ported from `core/src/config.ts`.
//! Loads opencode.json/opencode.jsonc from global, project, and .opencode directories.

pub mod agent;
pub mod provider;
pub mod command;
pub mod plugin;
pub mod reference;
pub mod experimental;
pub mod mcp;
pub mod lsp;
pub mod formatter;
pub mod compaction;
pub mod attachments;
pub mod tool_output;
pub mod watcher;
pub mod markdown;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

/// Root configuration for opencode.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_agent: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shell: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub autoupdate: Option<AutoUpdate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub share: Option<ShareMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snapshots: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agents: Option<HashMap<String, agent::AgentConfig>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub providers: Option<HashMap<String, provider::ProviderConfig>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commands: Option<HashMap<String, command::CommandConfig>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plugins: Option<HashMap<String, plugin::PluginConfig>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub references: Option<HashMap<String, reference::ReferenceConfig>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub experimental: Option<experimental::ExperimentalConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mcp: Option<HashMap<String, mcp::McpServer>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lsp: Option<lsp::LspConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub formatter: Option<formatter::FormatterConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compaction: Option<compaction::CompactionConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attachments: Option<attachments::AttachmentsConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_output: Option<tool_output::ToolOutputConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub watcher: Option<watcher::WatcherConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub markdown: Option<markdown::MarkdownConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skills: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AutoUpdate {
    Bool(bool),
    Notify,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ShareMode {
    Manual,
    Auto,
    Disabled,
}

/// A discovered config entry — either a document with data or a directory reference.
#[derive(Debug, Clone)]
pub enum ConfigEntry {
    Document { path: PathBuf, info: Config },
    Directory { path: PathBuf },
}

impl ConfigEntry {
    pub fn is_document(&self) -> bool {
        matches!(self, ConfigEntry::Document { .. })
    }
}

/// Load config from a file path.
pub async fn load(path: &Path) -> Result<Config, std::io::Error> {
    let content = tokio::fs::read_to_string(path).await?;
    serde_json::from_str(&content).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
}

/// Discover config files in the given directories.
/// Searches for `opencode.json` and `opencode.jsonc`.
pub async fn discover(
    directories: &[PathBuf],
) -> Vec<ConfigEntry> {
    let mut entries = Vec::new();
    for dir in directories {
        for name in &["opencode.json", "opencode.jsonc"] {
            let path = dir.join(name);
            if path.exists() {
                match tokio::fs::read_to_string(&path).await {
                    Ok(content) => {
                        let config: Result<Config, _> = serde_json::from_str(&content);
                        if let Ok(info) = config {
                            entries.push(ConfigEntry::Document { path: path.clone(), info });
                        }
                    }
                    Err(_) => continue,
                }
            }
        }
        entries.push(ConfigEntry::Directory { path: dir.clone() });
    }
    entries
}

/// Get the latest value for a key across config entries (highest priority last).
pub fn latest<K: Copy>(entries: &[ConfigEntry], accessor: impl Fn(&Config) -> Option<K>) -> Option<K> {
    entries
        .iter()
        .filter_map(|e| match e {
            ConfigEntry::Document { info, .. } => accessor(info),
            _ => None,
        })
        .last()
}
