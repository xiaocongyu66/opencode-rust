//! Permission system — four-stage pipeline (claude-code-book Ch04).
//!
//! Stage 1: validateInput (in Tool trait, run by runner before permission)
//! Stage 2: rule matching with deny > ask > allow precedence
//! Stage 3: checkPermissions (in Tool trait, context-aware)
//! Stage 4: interactive prompt (hook → classifier → user)
//!
//! The deny > ask > allow iron rule: when multiple rules match, deny wins
//! over ask wins over allow. This prevents an accidental `allow` from
//! overriding a deliberate `deny`.

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

    /// Stage 2: rule matching with deny > ask > allow precedence (Ch04).
    ///
    /// Scans all matching rules and returns the highest-precedence effect:
    /// - If any matching rule says Deny → Deny (cannot be overridden)
    /// - Else if any says Ask → Ask
    /// - Else if any says Allow → Allow
    /// - Else no match → Ask (default: be cautious)
    pub async fn check(&self, action: &str, resource: &str) -> PermissionEffect {
        let rules = self.rules.read().await;
        let mut has_allow = false;
        let mut has_ask = false;
        for rule in rules.iter() {
            if matches!(rule.action, ref a if a == action || a == "*")
                && matches!(rule.resource, ref r if r == resource || r == "*")
            {
                match rule.effect {
                    PermissionEffect::Deny => {
                        // Deny is absolute — return immediately (Ch04 iron rule).
                        return PermissionEffect::Deny;
                    }
                    PermissionEffect::Ask => has_ask = true,
                    PermissionEffect::Allow => has_allow = true,
                }
            }
        }
        if has_ask {
            PermissionEffect::Ask
        } else if has_allow {
            PermissionEffect::Allow
        } else {
            // No matching rules: default to Ask (principle of least privilege).
            PermissionEffect::Ask
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_deny_overrides_allow() {
        let sys = PermissionSystem::new();
        sys.add_rule(PermissionRule {
            action: "Bash".into(),
            resource: "*".into(),
            effect: PermissionEffect::Allow,
        }).await;
        sys.add_rule(PermissionRule {
            action: "Bash".into(),
            resource: "rm".into(),
            effect: PermissionEffect::Deny,
        }).await;
        // Deny wins over Allow for rm.
        assert!(matches!(sys.check("Bash", "rm").await, PermissionEffect::Deny));
        // Allow applies to other resources.
        assert!(matches!(sys.check("Bash", "ls").await, PermissionEffect::Allow));
    }

    #[tokio::test]
    async fn test_ask_overrides_allow() {
        let sys = PermissionSystem::new();
        sys.add_rule(PermissionRule {
            action: "Write".into(),
            resource: "*".into(),
            effect: PermissionEffect::Allow,
        }).await;
        sys.add_rule(PermissionRule {
            action: "Write".into(),
            resource: "/etc".into(),
            effect: PermissionEffect::Ask,
        }).await;
        // Ask wins over Allow for /etc.
        assert!(matches!(sys.check("Write", "/etc").await, PermissionEffect::Ask));
        // Allow for other paths.
        assert!(matches!(sys.check("Write", "/tmp").await, PermissionEffect::Allow));
    }

    #[tokio::test]
    async fn test_no_match_defaults_ask() {
        let sys = PermissionSystem::new();
        // No rules: default to Ask (principle of least privilege).
        assert!(matches!(sys.check("Bash", "ls").await, PermissionEffect::Ask));
    }

    #[tokio::test]
    async fn test_wildcard_match() {
        let sys = PermissionSystem::new();
        sys.add_rule(PermissionRule {
            action: "*".into(),
            resource: "*".into(),
            effect: PermissionEffect::Allow,
        }).await;
        assert!(matches!(sys.check("Anything", "anything").await, PermissionEffect::Allow));
    }
}
