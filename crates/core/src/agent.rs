//! Agent management.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use opencode_schema::ids::AgentID;
use opencode_schema::agent::AgentInfo;

pub struct AgentRegistry {
    agents: Arc<RwLock<HashMap<String, AgentInfo>>>,
}

impl AgentRegistry {
    pub fn new() -> Self {
        let mut agents = HashMap::new();
        agents.insert("build".to_string(), AgentInfo {
            id: AgentID::from_str("build"),
            model: None,
            request: Default::default(),
            system: None,
            description: Some("Default, full-access agent for development work".to_string()),
            mode: opencode_schema::agent::AgentMode::All,
            hidden: false,
            color: None,
            steps: None,
            permissions: Default::default(),
        });
        agents.insert("plan".to_string(), AgentInfo {
            id: AgentID::from_str("plan"),
            model: None,
            request: Default::default(),
            system: None,
            description: Some("Read-only agent for analysis and code exploration".to_string()),
            mode: opencode_schema::agent::AgentMode::Primary,
            hidden: false,
            color: None,
            steps: None,
            permissions: Default::default(),
        });
        Self { agents: Arc::new(RwLock::new(agents)) }
    }

    pub async fn get(&self, id: &str) -> Option<AgentInfo> {
        self.agents.read().await.get(id).cloned()
    }

    pub async fn list(&self) -> Vec<AgentInfo> {
        self.agents.read().await.values().cloned().collect()
    }

    pub async fn register(&self, info: AgentInfo) {
        self.agents.write().await.insert(info.id.0.clone(), info);
    }
}

impl Default for AgentRegistry {
    fn default() -> Self {
        Self::new()
    }
}
