//! Speculative classifier — Bash command risk assessment with 2s timeout.
//!
//! claude-code-book Ch04: a speculative classifier runs in parallel with the
//! interactive prompt. If it returns within 2s, its decision short-circuits
//! the prompt. If it times out, the user is asked normally.
//!
//! For now the classifier is a simple regex-based heuristic (no LLM call),
//! matching dangerous patterns like `rm -rf`, `:(){:|:&};:`, `dd of=/dev/`,
//! etc. An LLM-backed classifier can be added later as a provider call.

use std::time::Duration;

use crate::schema::permission::PermissionEffect;

/// Maximum time the classifier is allowed to run before we fall back to
/// asking the user (claude-code-book Ch04: 2 seconds).
pub const CLASSIFIER_TIMEOUT: Duration = Duration::from_secs(2);

/// Patterns that mark a shell command as definitely dangerous.
/// Matching any of these yields Deny (without consulting the user).
const DANGEROUS_PATTERNS: &[&str] = &[
    "rm -rf /",
    "rm -rf ~",
    "rm -rf $HOME",
    "rm -rf *",
    ":(){:|:&};:",           // fork bomb
    "dd of=/dev/sd",          // overwrite disk
    "dd of=/dev/null",
    "mkfs",
    "> /dev/sd",
    "chmod -R 777 /",
    "curl | sh",              // pipe-to-shell
    "curl | bash",
    "wget | sh",
    "wget | bash",
    "shutdown",
    "reboot",
    "halt",
    "init 0",
    "init 6",
];

/// Heuristic classification of a shell command. Returns the permission
/// effect to apply — Deny for dangerous, Ask for everything else.
///
/// This is the synchronous core; wrap it in `tokio::time::timeout` at the
/// call site to enforce the 2s budget (the heuristic itself is instant).
pub fn classify_bash(command: &str) -> PermissionEffect {
    let lower = command.to_lowercase();
    for pat in DANGEROUS_PATTERNS {
        if lower.contains(pat) {
            return PermissionEffect::Deny;
        }
    }
    // Everything else: ask the user (safer default).
    PermissionEffect::Ask
}

/// Run the classifier with a 2s timeout. In practice the heuristic is
/// synchronous and instant, so the timeout never fires — but the function
/// signature matches the LLM-backed variant we'd swap in later.
pub async fn classify_bash_timed(command: &str) -> PermissionEffect {
    let result = tokio::time::timeout(CLASSIFIER_TIMEOUT, async {
        classify_bash(command)
    }).await;
    match result {
        Ok(effect) => effect,
        Err(_) => {
            // Timed out — fall back to Ask.
            tracing::warn!("bash classifier timed out, falling back to Ask");
            PermissionEffect::Ask
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dangerous_commands_denied() {
        assert!(matches!(classify_bash("rm -rf /"), PermissionEffect::Deny));
        assert!(matches!(classify_bash("rm -rf ~"), PermissionEffect::Deny));
        assert!(matches!(classify_bash("rm -rf /home"), PermissionEffect::Deny));
        assert!(matches!(classify_bash(":(){:|:&};:"), PermissionEffect::Deny));
        assert!(matches!(classify_bash("dd of=/dev/sda"), PermissionEffect::Deny));
        assert!(matches!(classify_bash("mkfs.ext4 /dev/sda1"), PermissionEffect::Deny));
        assert!(matches!(classify_bash("shutdown -h now"), PermissionEffect::Deny));
    }

    #[test]
    fn test_safe_commands_ask() {
        assert!(matches!(classify_bash("ls -la"), PermissionEffect::Ask));
        assert!(matches!(classify_bash("echo hello"), PermissionEffect::Ask));
        assert!(matches!(classify_bash("git status"), PermissionEffect::Ask));
        // rm without -rf / is not matched by dangerous patterns
        assert!(matches!(classify_bash("rm file.txt"), PermissionEffect::Ask));
    }

    #[tokio::test]
    async fn test_timed_returns_within_budget() {
        // Heuristic is instant; should return well within 2s.
        let effect = classify_bash_timed("rm -rf /").await;
        assert!(matches!(effect, PermissionEffect::Deny));
    }
}
