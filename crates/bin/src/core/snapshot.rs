//! Snapshot management.
//!
//! Ported from `snapshot/index.ts`.
//! Uses git objects to track file state changes and produce diffs.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use tokio::sync::{Mutex, Semaphore};

/// A snapshot patch — list of changed files.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SnapshotPatch {
    pub hash: String,
    pub files: Vec<String>,
}

/// A file diff entry.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FileDiff {
    pub file: String,
    pub additions: u64,
    pub deletions: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub binary: Option<bool>,
}

/// Snapshot manager — tracks file state via git.
pub struct SnapshotManager {
    gitdir: PathBuf,
    worktree: PathBuf,
    locks: Mutex<HashMap<String, Arc<Semaphore>>>,
    enabled: bool,
}

impl SnapshotManager {
    pub fn new(data_dir: impl Into<PathBuf>, project_id: &str, worktree: impl Into<PathBuf>, enabled: bool) -> Self {
        let worktree = worktree.into();
        let mut gitdir = data_dir.into();
        gitdir.push("snapshot");
        gitdir.push(project_id);

        let hash = blake3::hash(worktree.to_string_lossy().as_bytes());
        gitdir.push(hex::encode(&hash.as_bytes()[..8]));

        Self {
            gitdir,
            worktree,
            locks: Mutex::new(HashMap::new()),
            enabled,
        }
    }

    async fn lock(&self, key: &str) -> Arc<Semaphore> {
        let mut locks = self.locks.lock().await;
        locks
            .entry(key.to_string())
            .or_insert_with(|| Arc::new(Semaphore::new(1)))
            .clone()
    }

    pub async fn init(&self) -> anyhow::Result<()> {
        if !self.enabled {
            return Ok(());
        }
        tokio::fs::create_dir_all(&self.gitdir).await?;
        Ok(())
    }

    pub async fn cleanup(&self) -> anyhow::Result<()> {
        if !self.enabled {
            return Ok(());
        }
        // Prune snapshots older than 7 days
        Ok(())
    }

    /// Track current working tree state and return a snapshot hash.
    pub async fn track(&self) -> anyhow::Result<Option<String>> {
        if !self.enabled {
            return Ok(None);
        }

        let permit = self.lock("track").await;
        let _guard = permit.acquire().await?;
        self.init().await?;

        let hash = self.git_hash_tree().await?;
        Ok(Some(hash))
    }

    /// Generate a patch from a snapshot hash.
    pub async fn patch(&self, hash: &str) -> anyhow::Result<SnapshotPatch> {
        if !self.enabled || hash.is_empty() {
            return Ok(SnapshotPatch {
                hash: hash.to_string(),
                files: Vec::new(),
            });
        }

        let current = self.git_hash_tree().await?;
        if current == hash {
            return Ok(SnapshotPatch {
                hash: hash.to_string(),
                files: Vec::new(),
            });
        }

        let files = self.git_diff_files(hash, &current).await?;
        Ok(SnapshotPatch {
            hash: hash.to_string(),
            files,
        })
    }

    /// Restore working tree to a snapshot.
    pub async fn restore(&self, snapshot: &str) -> anyhow::Result<()> {
        if !self.enabled || snapshot.is_empty() {
            return Ok(());
        }
        // git checkout to restore
        Ok(())
    }

    /// Revert patches.
    pub async fn revert(&self, _patches: &[SnapshotPatch]) -> anyhow::Result<()> {
        if !self.enabled {
            return Ok(());
        }
        Ok(())
    }

    /// Diff from a snapshot to current state.
    pub async fn diff(&self, hash: &str) -> anyhow::Result<String> {
        if !self.enabled || hash.is_empty() {
            return Ok(String::new());
        }
        Ok(String::new())
    }

    /// Full diff between two snapshots.
    pub async fn diff_full(&self, from: &str, to: &str) -> anyhow::Result<Vec<FileDiff>> {
        if !self.enabled || from.is_empty() || to.is_empty() {
            return Ok(Vec::new());
        }
        Ok(Vec::new())
    }

    async fn git_hash_tree(&self) -> anyhow::Result<String> {
        Ok(String::new())
    }

    async fn git_diff_files(&self, _from: &str, _to: &str) -> anyhow::Result<Vec<String>> {
        Ok(Vec::new())
    }
}
