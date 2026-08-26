//! Hook registry — 6-layer priority chain (claude-code-book Ch08).
//!
//! Priority (low → high): plugin < user < project < local < flag < policy.
//! Later layers override earlier ones; hooks at the same event+matcher
//! run in registration order within a layer.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

use super::protocol::HookInput;

/// Six configuration layers, lowest to highest priority.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HookLayer {
    Plugin,
    User,
    Project,
    Local,
    Flag,
    Policy,
}

impl Default for HookLayer {
    fn default() -> Self {
        Self::User
    }
}

/// A single hook definition. Mirrors claude-code-book Ch08 Command hook:
/// runs a shell command, gets JSON on stdin, returns JSON on stdout.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookConfig {
    /// Shell command to execute.
    pub command: String,
    /// Timeout in ms (default 2000).
    #[serde(default)]
    pub timeout_ms: u64,
    /// Status message shown while running.
    #[serde(default)]
    pub message: Option<String>,
    /// Run once then auto-remove.
    #[serde(default)]
    pub once: bool,
}

impl Default for HookConfig {
    fn default() -> Self {
        Self {
            command: String::new(),
            timeout_ms: 2000,
            message: None,
            once: false,
        }
    }
}

/// One matcher entry: "when event X targets tool Y, run these hooks".
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookEntry {
    /// Tool name or glob. Empty matches all tools.
    #[serde(default)]
    pub matcher: String,
    /// Hooks to run when the matcher hits.
    pub hooks: Vec<HookConfig>,
}

/// Layered registry of all hooks. Inner map: event_name → list of entries
/// (across layers). At dispatch time we flatten by layer priority.
#[derive(Default)]
pub struct HookRegistry {
    /// event → (layer → entries)
    entries: HashMap<String, HashMap<HookLayer, Vec<HookEntry>>>,
}

impl HookRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register hook entries for an event at a given layer.
    pub fn register(&mut self, event: &str, layer: HookLayer, entries: Vec<HookEntry>) {
        self.entries
            .entry(event.to_string())
            .or_default()
            .insert(layer, entries);
    }

    /// Load hooks for an event from a config file at the given layer.
    /// File format: `{ "<EventName>": [ { "matcher": "...", "hooks": [...] } ] }`
    pub fn load_file(&mut self, path: &PathBuf, layer: HookLayer) -> anyhow::Result<()> {
        let content = std::fs::read_to_string(path)?;
        let map: HashMap<String, Vec<HookEntry>> = serde_json::from_str(&content)?;
        for (event, entries) in map {
            self.register(&event, layer, entries);
        }
        Ok(())
    }

    /// Resolve the effective hook chain for `input` across all layers,
    /// ordered by layer priority then by registration order.
    pub fn resolve(&self, input: &HookInput) -> Vec<HookConfig> {
        let mut out: Vec<HookConfig> = Vec::new();
        let layers = [
            HookLayer::Plugin,
            HookLayer::User,
            HookLayer::Project,
            HookLayer::Local,
            HookLayer::Flag,
            HookLayer::Policy,
        ];
        if let Some(by_layer) = self.entries.get(&input.event) {
            for layer in layers {
                if let Some(entries) = by_layer.get(&layer) {
                    for entry in entries {
                        if matcher_hits(&entry.matcher, input.tool.as_deref()) {
                            for hook in &entry.hooks {
                                out.push(hook.clone());
                            }
                        }
                    }
                }
            }
        }
        out
    }
}

/// Simple matcher: empty pattern matches all; exact match; `*` glob.
fn matcher_hits(pattern: &str, tool: Option<&str>) -> bool {
    if pattern.is_empty() || pattern == "*" {
        return true;
    }
    match tool {
        Some(t) => pattern
            .split(',')
            .any(|p| p.trim() == t || p.trim() == "*"),
        None => false,
    }
}

/// Type alias to avoid deeply-nested generics in the global singleton.
pub type SharedRegistry = Arc<RwLock<HookRegistry>>;

/// Global singleton registry (lazy).
pub fn global() -> SharedRegistry {
    static REG: std::sync::OnceLock<SharedRegistry> = std::sync::OnceLock::new();
    REG.get_or_init(|| Arc::new(RwLock::new(HookRegistry::new())))
        .clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matcher_hits() {
        assert!(matcher_hits("", Some("Bash")));
        assert!(matcher_hits("*", Some("Bash")));
        assert!(matcher_hits("Bash", Some("Bash")));
        assert!(matcher_hits("Bash,Write", Some("Write")));
        assert!(!matcher_hits("Bash", Some("Write")));
        assert!(!matcher_hits("Bash", None));
        assert!(matcher_hits("", None)); // empty matches all, even no tool
    }

    #[test]
    fn test_layer_priority() {
        let mut reg = HookRegistry::new();
        reg.register(
            "PreToolUse",
            HookLayer::User,
            vec![HookEntry {
                matcher: "Bash".into(),
                hooks: vec![HookConfig {
                    command: "user-hook".into(),
                    timeout_ms: 1000,
                    message: None,
                    once: false,
                }],
            }],
        );
        reg.register(
            "PreToolUse",
            HookLayer::Policy,
            vec![HookEntry {
                matcher: "Bash".into(),
                hooks: vec![HookConfig {
                    command: "policy-hook".into(),
                    timeout_ms: 1000,
                    message: None,
                    once: false,
                }],
            }],
        ]);
        let input = HookInput {
            event: "PreToolUse".into(),
            tool: Some("Bash".into()),
            input: None,
            session_id: None,
            cwd: None,
        };
        let chain = reg.resolve(&input);
        assert_eq!(chain.len(), 2);
        // User comes before Policy in the layer order, regardless of registration.
        assert_eq!(chain[0].command, "user-hook");
        assert_eq!(chain[1].command, "policy-hook");
    }
}
