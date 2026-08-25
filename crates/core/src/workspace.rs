//! Workspace management — manages worktrees and working directories.

use std::sync::Arc;
use tokio::sync::RwLock;
use opencode_schema::ids::{ProjectID, WorkspaceID};
use opencode_schema::common::AbsolutePath;

#[derive(Debug, Clone)]
pub struct Workspace {
    pub id: WorkspaceID,
    pub project_id: ProjectID,
    pub directory: AbsolutePath,
}

pub struct WorkspaceManager {
    workspaces: Arc<RwLock<Vec<Workspace>>>,
}

impl WorkspaceManager {
    pub fn new() -> Self {
        Self { workspaces: Arc::new(RwLock::new(vec![])) }
    }

    pub async fn create(&self, project_id: ProjectID, directory: AbsolutePath) -> Workspace {
        let ws = Workspace {
            id: WorkspaceID::new(),
            project_id,
            directory,
        };
        self.workspaces.write().await.push(ws.clone());
        ws
    }

    pub async fn list(&self) -> Vec<Workspace> {
        self.workspaces.read().await.clone()
    }

    pub async fn get(&self, id: &WorkspaceID) -> Option<Workspace> {
        self.workspaces.read().await.iter().find(|w| w.id == *id).cloned()
    }
}

impl Default for WorkspaceManager {
    fn default() -> Self {
        Self::new()
    }
}
