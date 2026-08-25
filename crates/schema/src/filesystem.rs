//! Filesystem data models.

use serde::{Deserialize, Serialize};

use crate::common::RelativePath;

/// Filesystem entry type.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileSystemEntryType {
    File,
    Directory,
}

/// A filesystem entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSystemEntry {
    pub path: RelativePath,
    #[serde(rename = "type")]
    pub entry_type: FileSystemEntryType,
}

/// A grep submatch.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSystemSubmatch {
    pub text: String,
    pub start: u64,
    pub end: u64,
}

/// A grep match.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSystemMatch {
    pub entry: FileSystemEntry,
    pub line: u64,
    pub offset: u64,
    pub text: String,
    pub submatches: Vec<FileSystemSubmatch>,
}

/// Find input.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSystemFindInput {
    pub query: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "type")]
    pub find_type: Option<FileSystemEntryType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u64>,
}
