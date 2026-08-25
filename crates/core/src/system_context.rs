//! System context — context algebra for building LLM prompts.
//!
//! Manages context sources, epochs, and context selection for sessions.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// A context source that contributes to the system prompt.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContextSource {
    pub name: String,
    pub priority: i32,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_type: Option<String>,
}

/// A context epoch — a point-in-time snapshot of active context sources.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContextEpoch {
    pub seq: u64,
    pub sources: Vec<ContextSource>,
}

pub struct SystemContext {
    sources: HashMap<String, ContextSource>,
    current_epoch: ContextEpoch,
}

impl SystemContext {
    pub fn new() -> Self {
        Self {
            sources: HashMap::new(),
            current_epoch: ContextEpoch { seq: 0, sources: vec![] },
        }
    }

    pub fn register(&mut self, source: ContextSource) {
        self.sources.insert(source.name.clone(), source);
        self.current_epoch.seq += 1;
        self.rebuild_epoch();
    }

    pub fn unregister(&mut self, name: &str) {
        self.sources.remove(name);
        self.current_epoch.seq += 1;
        self.rebuild_epoch();
    }

    pub fn epoch(&self) -> &ContextEpoch {
        &self.current_epoch
    }

    pub fn build_prompt(&self) -> String {
        self.current_epoch.sources
            .iter()
            .map(|s| s.content.clone())
            .collect::<Vec<_>>()
            .join("\n\n")
    }

    fn rebuild_epoch(&mut self) {
        let mut sources: Vec<ContextSource> = self.sources.values().cloned().collect();
        sources.sort_by(|a, b| b.priority.cmp(&a.priority));
        self.current_epoch.sources = sources;
    }
}

impl Default for SystemContext {
    fn default() -> Self {
        Self::new()
    }
}
