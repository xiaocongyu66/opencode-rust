//! Agent management.
//!
//! Ported from `core/src/agent.ts`.
//! Manages agent definitions, selection, and default resolution.
//! The TS version uses a State<Transformable> pattern; here we use
//! a `RwLock<HashMap>` with the same business logic.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::schema::agent::{AgentInfo, AgentMode};
use crate::schema::ids::AgentID;

pub const DEFAULT_AGENT_ID: &str = "build";

#[derive(Debug, Clone)]
pub struct AgentSelection {
    pub id: AgentID,
    pub info: Option<AgentInfo>,
}

pub struct AgentRegistry {
    agents: Arc<RwLock<HashMap<String, AgentInfo>>>,
    default_id: Arc<RwLock<Option<String>>>,
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
            mode: AgentMode::All,
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
            mode: AgentMode::Primary,
            hidden: false,
            color: None,
            steps: None,
            permissions: Default::default(),
        });
        Self {
            agents: Arc::new(RwLock::new(agents)),
            default_id: Arc::new(RwLock::new(None)),
        }
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

    pub async fn remove(&self, id: &str) {
        self.agents.write().await.remove(id);
    }

    pub async fn set_default(&self, id: &str) {
        *self.default_id.write().await = Some(id.to_string());
    }

    fn is_selectable(agent: &AgentInfo) -> bool {
        agent.mode != AgentMode::Subagent && !agent.hidden
    }

    /// Resolve the default agent following the TS logic:
    /// 1. Check configured default
    /// 2. Check "build" agent
    /// 3. Return first selectable agent
    pub async fn default_agent(&self) -> Option<AgentInfo> {
        let agents = self.agents.read().await;
        let default_id = self.default_id.read().await;

        if let Some(id) = default_id.as_ref() {
            if let Some(agent) = agents.get(id) {
                if Self::is_selectable(agent) {
                    return Some(agent.clone());
                }
            }
        }

        if let Some(build) = agents.get("build") {
            if Self::is_selectable(build) {
                return Some(build.clone());
            }
        }

        for agent in agents.values() {
            if Self::is_selectable(agent) {
                return Some(agent.clone());
            }
        }

        None
    }

    /// Resolve an agent by optional ID or string.
    /// If `id` is `None`, returns the default agent.
    pub async fn resolve(&self, id: Option<&str>) -> Option<AgentInfo> {
        match id {
            Some(id) => self.get(id).await,
            None => self.default_agent().await,
        }
    }

    /// Select an agent, returning a Selection with ID and optional info.
    pub async fn select(&self, id: Option<&str>) -> AgentSelection {
        match id {
            Some(id) => AgentSelection {
                id: AgentID::from_str(id),
                info: self.get(id).await,
            },
            None => {
                let info = self.default_agent().await;
                AgentSelection {
                    id: info.as_ref()
                        .map(|a| a.id.clone())
                        .unwrap_or_else(|| AgentID::from_str(DEFAULT_AGENT_ID)),
                    info,
                }
            }
        }
    }
}

impl Default for AgentRegistry {
    fn default() -> Self {
        Self::new()
    }
}
