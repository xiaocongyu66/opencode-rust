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

// ============================================================================
// Six-layer config priority (claude-code-book Ch05)
// ============================================================================

/// Configuration layers, lowest to highest priority. Later layers
/// override earlier ones via deep merge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConfigLayer {
    /// Bundled plugin defaults (base layer).
    Plugin,
    /// ~/.rsopencode/config.toml — user-wide defaults.
    User,
    /// .rsopencode/config.toml — project-shared, checked into git.
    Project,
    /// .rsopencode/config.local.toml — project-local, gitignored.
    Local,
    /// CLI flags (--model, --agent, etc.) — one-shot override.
    Flag,
    /// /etc/rsopencode/policy.toml — enterprise lockdown, highest priority.
    Policy,
}

impl ConfigLayer {
    /// Load order: lowest priority first, highest last.
    pub const ALL: [Self; 6] = [
        Self::Plugin,
        Self::User,
        Self::Project,
        Self::Local,
        Self::Flag,
        Self::Policy,
    ];

    /// File path for this layer, given the user's home and cwd.
    pub fn path(&self, home: &Path, cwd: &Path) -> Option<PathBuf> {
        match self {
            Self::Plugin => None, // plugins carry their own config
            Self::User => Some(home.join(".rsopencode").join("config.toml")),
            Self::Project => Some(cwd.join(".rsopencode").join("config.toml")),
            Self::Local => Some(cwd.join(".rsopencode").join("config.local.toml")),
            Self::Flag => None, // CLI flags are injected in-memory
            Self::Policy => Some(PathBuf::from("/etc/rsopencode/policy.toml")),
        }
    }
}

/// Deep-merge `overlay` into `base`. Non-None fields in overlay replace
/// base's; nested maps merge recursively; lists concatenate.
///
/// This is the Ch05 "deep merge with custom rules" — not a blind replace.
pub fn merge(base: &mut Config, overlay: Config) {
    // Option<T>: take from overlay if present.
    if overlay.model.is_some() {
        base.model = overlay.model;
    }
    if overlay.default_agent.is_some() {
        base.default_agent = overlay.default_agent;
    }
    if overlay.shell.is_some() {
        base.shell = overlay.shell;
    }
    if overlay.autoupdate.is_some() {
        base.autoupdate = overlay.autoupdate;
    }
    if overlay.share.is_some() {
        base.share = overlay.share;
    }
    if overlay.username.is_some() {
        base.username = overlay.username;
    }
    if overlay.snapshots.is_some() {
        base.snapshots = overlay.snapshots;
    }
    // HashMap fields: merge key-by-key (overlay wins on key collision).
    merge_map(&mut base.agents, overlay.agents);
    merge_map(&mut base.providers, overlay.providers);
    merge_map(&mut base.commands, overlay.commands);
    merge_map(&mut base.plugins, overlay.plugins);
    merge_map(&mut base.references, overlay.references);
    merge_map(&mut base.mcp, overlay.mcp);
    // Vec fields: concatenate (plugin skills + user skills).
    if let Some(extra) = overlay.skills {
        base.skills = Some(base.skills.take().unwrap_or_default().into_iter().chain(extra).collect());
    }
    if let Some(extra) = overlay.instructions {
        base.instructions = Some(base.instructions.take().unwrap_or_default().into_iter().chain(extra).collect());
    }
}

fn merge_map<V>(base: &mut Option<HashMap<String, V>>, overlay: Option<HashMap<String, V>>) {
    if let Some(over) = overlay {
        let map = base.get_or_insert_with(HashMap::new);
        for (k, v) in over {
            map.insert(k, v);
        }
    }
}

/// Load and merge all six layers in priority order (Ch05).
///
/// Missing files are silently skipped (that layer contributes nothing).
/// Returns the merged Config, or Default if no layer had a file.
pub async fn load_merged(home: &Path, cwd: &Path) -> Config {
    let mut merged = Config::default();
    for layer in ConfigLayer::ALL {
        if let Some(path) = layer.path(home, cwd) {
            if !path.exists() {
                continue;
            }
            match load(&path).await {
                Ok(cfg) => merge(&mut merged, cfg),
                Err(e) => {
                    tracing::warn!("config layer {:?} load failed ({}): {}", layer, path.display(), e);
                }
            }
        }
    }
    merged
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_options() {
        let mut base = Config::default();
        let overlay = Config {
            model: Some("gpt-4".into()),
            shell: Some("/bin/zsh".into()),
            ..Default::default()
        };
        merge(&mut base, overlay);
        assert_eq!(base.model.as_deref(), Some("gpt-4"));
        assert_eq!(base.shell.as_deref(), Some("/bin/zsh"));
    }

    #[test]
    fn test_merge_maps() {
        let mut base = Config::default();
        let mut agents = HashMap::new();
        agents.insert("build".to_string(), agent::AgentConfig::default());
        let overlay = Config {
            agents: Some(agents),
            ..Default::default()
        };
        merge(&mut base, overlay);
        assert!(base.agents.as_ref().map_or(false, |m| m.contains_key("build")));
    }

    #[test]
    fn test_layer_order() {
        // Higher-priority layers come later in ALL.
        let order: Vec<_> = ConfigLayer::ALL.iter().collect();
        assert_eq!(order[0], &ConfigLayer::Plugin);
        assert_eq!(order[5], &ConfigLayer::Policy);
    }
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
