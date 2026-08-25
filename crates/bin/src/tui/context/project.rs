use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PathInfo {
    #[serde(default)]
    pub home: String,
    #[serde(default)]
    pub state: String,
    #[serde(default)]
    pub config: String,
    #[serde(default)]
    pub worktree: String,
    #[serde(default)]
    pub directory: String,
}

#[derive(Debug, Clone, Default)]
pub struct WorkspaceInfo {
    pub id: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum WorkspaceStatus {
    Connected,
    Connecting,
    Disconnected,
    Error,
}

#[derive(Default)]
pub struct ProjectStore {
    pub project_id: Option<String>,
    pub project_worktree: Option<String>,
    pub project_main_dir: Option<String>,
    pub instance_path: PathInfo,
    pub workspace_current: Option<String>,
    pub workspace_list: Vec<WorkspaceInfo>,
    pub workspace_status: HashMap<String, WorkspaceStatus>,
}

pub struct ProjectContext {
    pub data: Arc<Mutex<ProjectStore>>,
    pub directory: Option<String>,
}

impl ProjectContext {
    pub fn new(directory: Option<String>) -> Self {
        let instance_path = PathInfo {
            directory: directory.clone().unwrap_or_default(),
            ..Default::default()
        };
        Self {
            data: Arc::new(Mutex::new(ProjectStore {
                instance_path,
                ..Default::default()
            })),
            directory,
        }
    }

    pub fn project(&self) -> Option<String> {
        self.data.lock().unwrap().project_id.clone()
    }

    pub fn instance_path(&self) -> PathInfo {
        self.data.lock().unwrap().instance_path.clone()
    }

    pub fn instance_directory(&self) -> Option<String> {
        let dir = self.data.lock().unwrap().instance_path.directory.clone();
        if dir.is_empty() { None } else { Some(dir) }
    }

    pub fn workspace_current(&self) -> Option<String> {
        self.data.lock().unwrap().workspace_current.clone()
    }

    pub fn workspace_set(&self, next: Option<String>) {
        let mut store = self.data.lock().unwrap();
        if store.workspace_current == next {
            return;
        }
        store.workspace_current = next;
    }

    pub fn workspace_list(&self) -> Vec<WorkspaceInfo> {
        self.data.lock().unwrap().workspace_list.clone()
    }

    pub fn workspace_status(&self, workspace_id: &str) -> Option<WorkspaceStatus> {
        self.data
            .lock()
            .unwrap()
            .workspace_status
            .get(workspace_id)
            .cloned()
    }

    pub fn set_instance_path(&self, path: PathInfo) {
        self.data.lock().unwrap().instance_path = path;
    }

    pub fn set_project(&self, id: Option<String>, worktree: Option<String>, main_dir: Option<String>) {
        let mut store = self.data.lock().unwrap();
        store.project_id = id;
        store.project_worktree = worktree;
        store.project_main_dir = main_dir;
    }

    pub fn set_workspace_list(&self, list: Vec<WorkspaceInfo>) {
        self.data.lock().unwrap().workspace_list = list;
    }

    pub fn set_workspace_status(&self, workspace_id: &str, status: WorkspaceStatus) {
        self.data
            .lock()
            .unwrap()
            .workspace_status
            .insert(workspace_id.to_string(), status);
    }
}
