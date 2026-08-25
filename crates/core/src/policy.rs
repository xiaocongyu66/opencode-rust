//! Policy engine — controls tool execution permissions.

use opencode_schema::permission::{PermissionEffect, PermissionRule};
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct PolicyEngine {
    rules: Arc<RwLock<Vec<PermissionRule>>>,
}

impl PolicyEngine {
    pub fn new() -> Self {
        Self { rules: Arc::new(RwLock::new(vec![])) }
    }

    pub async fn evaluate(&self, action: &str, resource: &str) -> PermissionEffect {
        let rules = self.rules.read().await;
        for rule in rules.iter() {
            if Self::matches(&rule.action, action) && Self::matches(&rule.resource, resource) {
                return rule.effect.clone();
            }
        }
        PermissionEffect::Ask
    }

    pub async fn add_rule(&self, rule: PermissionRule) {
        self.rules.write().await.push(rule);
    }

    fn matches(pattern: &str, value: &str) -> bool {
        if pattern == "*" { return true; }
        if pattern.ends_with("/*") {
            let prefix = &pattern[..pattern.len() - 2];
            return value.starts_with(prefix);
        }
        pattern == value
    }
}

impl Default for PolicyEngine {
    fn default() -> Self {
        Self::new()
    }
}
