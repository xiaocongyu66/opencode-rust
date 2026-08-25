//! Tool output store — persists tool execution outputs.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use opencode_schema::llm::ToolContent;

#[derive(Debug, Clone, Default)]
pub struct ToolOutput {
    pub structured: serde_json::Value,
    pub content: Vec<ToolContent>,
    pub output_paths: Vec<String>,
}

pub struct ToolOutputStore {
    store: Arc<RwLock<HashMap<String, ToolOutput>>>,
}

impl ToolOutputStore {
    pub fn new() -> Self {
        Self { store: Arc::new(RwLock::new(HashMap::new())) }
    }

    pub async fn save(&self, call_id: &str, output: ToolOutput) {
        self.store.write().await.insert(call_id.to_string(), output);
    }

    pub async fn get(&self, call_id: &str) -> Option<ToolOutput> {
        self.store.read().await.get(call_id).cloned()
    }

    pub async fn delete(&self, call_id: &str) {
        self.store.write().await.remove(call_id);
    }
}

impl Default for ToolOutputStore {
    fn default() -> Self {
        Self::new()
    }
}
