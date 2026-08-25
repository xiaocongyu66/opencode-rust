//! Process management.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct ProcessManager {
    processes: Arc<RwLock<HashMap<u32, tokio::process::Child>>>,
}

impl ProcessManager {
    pub fn new() -> Self {
        Self { processes: Arc::new(RwLock::new(HashMap::new())) }
    }

    pub async fn spawn(&self, command: String, args: Vec<String>) -> Result<u32, std::io::Error> {
        let child = tokio::process::Command::new(&command)
            .args(&args)
            .spawn()?;
        let pid = child.id().unwrap_or(0);
        self.processes.write().await.insert(pid, child);
        Ok(pid)
    }

    pub async fn kill(&self, pid: u32) -> Result<(), std::io::Error> {
        if let Some(mut child) = self.processes.write().await.remove(&pid) {
            child.kill().await?;
        }
        Ok(())
    }
}

impl Default for ProcessManager {
    fn default() -> Self {
        Self::new()
    }
}
