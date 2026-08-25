//! Project management.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use opencode_schema::ids::ProjectID;
use opencode_schema::project::ProjectInfo;

pub struct ProjectRegistry {
    projects: Arc<RwLock<HashMap<String, ProjectInfo>>>,
}

impl ProjectRegistry {
    pub fn new() -> Self {
        let mut projects = HashMap::new();
        projects.insert("global".to_string(), ProjectInfo {
            id: ProjectID::global(),
            worktree: std::env::current_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default(),
            vcs: None,
            name: None,
            icon: None,
            commands: None,
            time: Default::default(),
            sandboxes: vec![],
        });
        Self { projects: Arc::new(RwLock::new(projects)) }
    }

    pub async fn get(&self, id: &ProjectID) -> Option<ProjectInfo> {
        self.projects.read().await.get(id.as_str()).cloned()
    }

    pub async fn list(&self) -> Vec<ProjectInfo> {
        self.projects.read().await.values().cloned().collect()
    }

    pub async fn register(&self, info: ProjectInfo) {
        self.projects.write().await.insert(info.id.0.clone(), info);
    }
}

impl Default for ProjectRegistry {
    fn default() -> Self {
        Self::new()
    }
}
