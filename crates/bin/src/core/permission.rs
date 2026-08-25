//! Permission system.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::schema::ids::PermissionID;
use crate::schema::permission::{PermissionEffect, PermissionReply, PermissionRule};

pub struct PermissionSystem {
    rules: Arc<RwLock<Vec<PermissionRule>>>,
    saved: Arc<RwLock<HashMap<String, PermissionReply>>>,
}

impl PermissionSystem {
    pub fn new() -> Self {
        Self {
            rules: Arc::new(RwLock::new(Vec::new())),
            saved: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn check(&self, action: &str, resource: &str) -> PermissionEffect {
        let rules = self.rules.read().await;
        for rule in rules.iter() {
            if rule.action == action && rule.resource == resource {
                return rule.effect.clone();
            }
        }
        PermissionEffect::Ask
    }

    pub async fn add_rule(&self, rule: PermissionRule) {
        self.rules.write().await.push(rule);
    }

    pub async fn save_reply(&self, id: PermissionID, reply: PermissionReply) {
        self.saved.write().await.insert(id.0, reply);
    }

    pub async fn get_saved_reply(&self, id: &PermissionID) -> Option<PermissionReply> {
        self.saved.read().await.get(&id.0).cloned()
    }
}

impl Default for PermissionSystem {
    fn default() -> Self {
        Self::new()
    }
}
