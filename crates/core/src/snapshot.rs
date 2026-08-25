//! Snapshot management — captures and restores session file states.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use opencode_schema::common::RelativePath;
use opencode_schema::revert::RevertFileDiff;

#[derive(Debug, Clone, Default)]
pub struct Snapshot {
    pub files: HashMap<String, String>,
    pub diffs: Vec<RevertFileDiff>,
}

pub struct SnapshotManager {
    snapshots: Arc<RwLock<HashMap<String, Snapshot>>>,
}

impl SnapshotManager {
    pub fn new() -> Self {
        Self { snapshots: Arc::new(RwLock::new(HashMap::new())) }
    }

    pub async fn capture(&self, id: &str, base_dir: &str, paths: &[RelativePath]) -> Snapshot {
        let mut files = HashMap::new();
        for path in paths {
            let full = format!("{}/{}", base_dir, path);
            if let Ok(content) = tokio::fs::read_to_string(&full).await {
                files.insert(path.to_string(), content);
            }
        }
        let snapshot = Snapshot { files, diffs: vec![] };
        self.snapshots.write().await.insert(id.to_string(), snapshot.clone());
        snapshot
    }

    pub async fn get(&self, id: &str) -> Option<Snapshot> {
        self.snapshots.read().await.get(id).cloned()
    }

    pub async fn restore(&self, id: &str, base_dir: &str) -> Result<(), String> {
        let snapshot = self.get(id).await.ok_or("Snapshot not found")?;
        for (path, content) in &snapshot.files {
            let full = format!("{}/{}", base_dir, path);
            if let Some(parent) = std::path::Path::new(&full).parent() {
                let _ = tokio::fs::create_dir_all(parent).await;
            }
            tokio::fs::write(&full, content).await.map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    pub async fn delete(&self, id: &str) {
        self.snapshots.write().await.remove(id);
    }
}

impl Default for SnapshotManager {
    fn default() -> Self {
        Self::new()
    }
}
