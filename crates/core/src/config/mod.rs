//! Configuration management.

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
use serde::{Deserialize, Serialize};

/// Root configuration for opencode.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent: Option<HashMap<String, agent::AgentConfig>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<HashMap<String, provider::ProviderConfig>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<HashMap<String, command::CommandConfig>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plugin: Option<HashMap<String, plugin::PluginConfig>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference: Option<HashMap<String, reference::ReferenceConfig>>,
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
}

/// Load config from a file path.
pub async fn load(path: &str) -> Result<Config, std::io::Error> {
    let content = tokio::fs::read_to_string(path).await?;
    if path.ends_with(".json") {
        serde_json::from_str(&content).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    } else {
        // Try JSON first, then YAML
        serde_json::from_str(&content)
            .or_else(|_| {
                // Fallback: try parsing as JSON even without extension
                serde_json::from_str(&content)
            })
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }
}
