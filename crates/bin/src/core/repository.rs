//! Repository management — git repo detection and caching.

use std::path::Path;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct RepositoryCache {
    cache: Arc<RwLock<HashMap<String, RepoInfo>>>,
}

#[derive(Debug, Clone)]
pub struct RepoInfo {
    pub path: String,
    pub branch: String,
    pub is_repo: bool,
}

impl RepositoryCache {
    pub fn new() -> Self {
        Self { cache: Arc::new(RwLock::new(HashMap::new())) }
    }

    pub async fn get_or_detect(&self, path: &Path) -> RepoInfo {
        let key = path.to_string_lossy().to_string();
        {
            let cache = self.cache.read().await;
            if let Some(info) = cache.get(&key) {
                return info.clone();
            }
        }

        let is_repo = path.join(".git").exists();
        let branch = if is_repo {
            crate::core::git::Git::current_branch(path).await.unwrap_or_else(|_| "unknown".to_string())
        } else {
            "none".to_string()
        };

        let info = RepoInfo {
            path: key.clone(),
            branch,
            is_repo,
        };
        self.cache.write().await.insert(key, info.clone());
        info
    }
}

impl Default for RepositoryCache {
    fn default() -> Self {
        Self::new()
    }
}
