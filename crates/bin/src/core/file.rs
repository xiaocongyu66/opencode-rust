//! File operations — file read/write/patch helpers.

use crate::schema::common::{AbsolutePath, RelativePath};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileMutation {
    pub path: RelativePath,
    #[serde(rename = "type")]
    pub mutation_type: MutationType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub old: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MutationType {
    Create,
    Edit,
    Delete,
}

pub struct FileOps;

impl FileOps {
    pub async fn read(path: &AbsolutePath) -> Result<String, std::io::Error> {
        tokio::fs::read_to_string(path.as_str()).await
    }

    pub async fn write(path: &AbsolutePath, content: &str) -> Result<(), std::io::Error> {
        if let Some(parent) = std::path::Path::new(path.as_str()).parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        tokio::fs::write(path.as_str(), content).await
    }

    pub async fn edit(path: &AbsolutePath, old: &str, new: &str) -> Result<(), std::io::Error> {
        let content = tokio::fs::read_to_string(path.as_str()).await?;
        let new_content = content.replacen(old, new, 1);
        tokio::fs::write(path.as_str(), new_content).await
    }

    pub async fn delete(path: &AbsolutePath) -> Result<(), std::io::Error> {
        tokio::fs::remove_file(path.as_str()).await
    }

    pub async fn exists(path: &AbsolutePath) -> bool {
        tokio::fs::metadata(path.as_str()).await.is_ok()
    }
}
