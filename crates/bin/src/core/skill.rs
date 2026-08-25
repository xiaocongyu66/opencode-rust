//! Skill system.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::schema::skill::SkillInfo;

pub struct SkillRegistry {
    skills: Arc<RwLock<HashMap<String, SkillInfo>>>,
}

impl SkillRegistry {
    pub fn new() -> Self {
        Self { skills: Arc::new(RwLock::new(HashMap::new())) }
    }

    pub async fn get(&self, name: &str) -> Option<SkillInfo> {
        self.skills.read().await.get(name).cloned()
    }

    pub async fn list(&self) -> Vec<SkillInfo> {
        self.skills.read().await.values().cloned().collect()
    }

    pub async fn register(&self, info: SkillInfo) {
        self.skills.write().await.insert(info.name.clone(), info);
    }
}

impl Default for SkillRegistry {
    fn default() -> Self {
        Self::new()
    }
}
