//! Storage abstraction.
//!
//! Ported from `storage/storage.ts`.
//! Provides JSON-file-based key-value storage with read/write/update/list.

use std::path::PathBuf;

use anyhow::Result;
use tokio::sync::RwLock;

/// Storage error.
#[derive(Debug, Clone, thiserror::Error)]
pub enum StorageError {
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("FS error: {0}")]
    Fs(String),
}

/// Storage interface — JSON file-based key-value store.
pub struct Storage {
    base_dir: PathBuf,
    lock: RwLock<()>,
}

impl Storage {
    pub fn new(base_dir: impl Into<PathBuf>) -> Self {
        Self {
            base_dir: base_dir.into(),
            lock: RwLock::new(()),
        }
    }

    fn file_path(&self, key: &[String]) -> PathBuf {
        let mut path = self.base_dir.clone();
        for k in key {
            path.push(k);
        }
        path.set_extension("json");
        path
    }

    pub async fn read<T: serde::de::DeserializeOwned>(&self, key: &[String]) -> Result<T, StorageError> {
        let path = self.file_path(key);
        let content = tokio::fs::read_to_string(&path)
            .await
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    StorageError::NotFound(path.to_string_lossy().to_string())
                } else {
                    StorageError::Fs(e.to_string())
                }
            })?;
        serde_json::from_str(&content).map_err(|e| StorageError::Fs(e.to_string()))
    }

    pub async fn write<T: serde::Serialize>(&self, key: &[String], content: &T) -> Result<(), StorageError> {
        let _guard = self.lock.write().await;
        let path = self.file_path(key);
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| StorageError::Fs(e.to_string()))?;
        }
        let json = serde_json::to_string_pretty(content).map_err(|e| StorageError::Fs(e.to_string()))?;
        tokio::fs::write(&path, json)
            .await
            .map_err(|e| StorageError::Fs(e.to_string()))?;
        Ok(())
    }

    pub async fn update<T: serde::de::DeserializeOwned + serde::Serialize, F>(
        &self,
        key: &[String],
        f: F,
    ) -> Result<T, StorageError>
    where
        F: FnOnce(&mut T),
    {
        let _guard = self.lock.write().await;
        let path = self.file_path(key);

        let mut value: T = match tokio::fs::read_to_string(&path).await {
            Ok(content) => serde_json::from_str(&content).map_err(|e| StorageError::Fs(e.to_string()))?,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                return Err(StorageError::NotFound(path.to_string_lossy().to_string()));
            }
            Err(e) => return Err(StorageError::Fs(e.to_string())),
        };

        f(&mut value);

        let json = serde_json::to_string_pretty(&value).map_err(|e| StorageError::Fs(e.to_string()))?;
        tokio::fs::write(&path, json)
            .await
            .map_err(|e| StorageError::Fs(e.to_string()))?;

        Ok(value)
    }

    pub async fn remove(&self, key: &[String]) -> Result<(), StorageError> {
        let path = self.file_path(key);
        tokio::fs::remove_file(&path)
            .await
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    StorageError::NotFound(path.to_string_lossy().to_string())
                } else {
                    StorageError::Fs(e.to_string())
                }
            })?;
        Ok(())
    }

    pub async fn list(&self, prefix: &[String]) -> Result<Vec<Vec<String>>, StorageError> {
        let mut path = self.base_dir.clone();
        for p in prefix {
            path.push(p);
        }

        if !path.exists() {
            return Ok(Vec::new());
        }

        let mut results = Vec::new();
        let mut stack = vec![path.clone()];

        while let Some(dir) = stack.pop() {
            let mut entries = tokio::fs::read_dir(&dir)
                .await
                .map_err(|e| StorageError::Fs(e.to_string()))?;

            while let Some(entry) = entries
                .next_entry()
                .await
                .map_err(|e| StorageError::Fs(e.to_string()))?
            {
                let entry_path = entry.path();
                if entry_path.is_dir() {
                    stack.push(entry_path);
                } else if entry_path.extension().and_then(|e| e.to_str()) == Some("json") {
                    let relative = entry_path
                        .strip_prefix(&self.base_dir)
                        .unwrap_or(&entry_path);
                    let key: Vec<String> = relative
                        .components()
                        .filter_map(|c| c.as_os_str().to_str().map(|s| s.to_string()))
                        .collect();
                    let key: Vec<String> = key
                        .into_iter()
                        .map(|mut k| {
                            if k.ends_with(".json") {
                                k.truncate(k.len() - 5);
                            }
                            k
                        })
                        .collect();
                    results.push(key);
                }
            }
        }

        Ok(results)
    }
}
