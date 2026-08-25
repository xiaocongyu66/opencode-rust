//! Git operations.

use std::path::Path;

pub struct Git;

impl Git {
    pub async fn is_repo(path: &Path) -> bool {
        path.join(".git").exists()
    }

    pub async fn current_branch(path: &Path) -> Result<String, std::io::Error> {
        let head_path = path.join(".git").join("HEAD");
        let head = tokio::fs::read_to_string(&head_path).await?;
        let branch = head
            .trim()
            .strip_prefix("ref: refs/heads/")
            .unwrap_or("detached");
        Ok(branch.to_string())
    }

    pub async fn status(path: &Path) -> Result<String, std::io::Error> {
        let output = tokio::process::Command::new("git")
            .arg("status")
            .arg("--short")
            .current_dir(path)
            .output()
            .await?;
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    pub async fn diff(path: &Path) -> Result<String, std::io::Error> {
        let output = tokio::process::Command::new("git")
            .arg("diff")
            .current_dir(path)
            .output()
            .await?;
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}
