//! Filesystem operations.

use opencode_schema::common::{AbsolutePath, RelativePath};
use opencode_schema::filesystem::{FileSystemEntry, FileSystemEntryType};

pub struct FileSystem;

impl FileSystem {
    pub async fn read_file(path: &AbsolutePath) -> Result<String, std::io::Error> {
        tokio::fs::read_to_string(path.as_str()).await
    }

    pub async fn write_file(path: &AbsolutePath, content: &str) -> Result<(), std::io::Error> {
        if let Some(parent) = std::path::Path::new(path.as_str()).parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        tokio::fs::write(path.as_str(), content).await
    }

    pub async fn list_dir(path: &AbsolutePath) -> Result<Vec<FileSystemEntry>, std::io::Error> {
        let mut entries = Vec::new();
        let mut reader = tokio::fs::read_dir(path.as_str()).await?;
        while let Some(entry) = reader.next_entry().await? {
            let name = entry.file_name().to_string_lossy().to_string();
            let ftype = entry.file_type().await?;
            let entry_type = if ftype.is_dir() {
                FileSystemEntryType::Directory
            } else {
                FileSystemEntryType::File
            };
            entries.push(FileSystemEntry {
                path: RelativePath::new(name),
                entry_type,
            });
        }
        entries.sort_by(|a, b| a.path.cmp(&b.path));
        Ok(entries)
    }

    pub async fn exists(path: &AbsolutePath) -> bool {
        tokio::fs::metadata(path.as_str()).await.is_ok()
    }

    pub async fn create_dir(path: &AbsolutePath) -> Result<(), std::io::Error> {
        tokio::fs::create_dir_all(path.as_str()).await
    }

    pub async fn remove(path: &AbsolutePath) -> Result<(), std::io::Error> {
        let metadata = tokio::fs::metadata(path.as_str()).await?;
        if metadata.is_dir() {
            tokio::fs::remove_dir_all(path.as_str()).await
        } else {
            tokio::fs::remove_file(path.as_str()).await
        }
    }
}
