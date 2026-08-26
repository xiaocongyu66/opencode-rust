//! Hook executor — runs Command hooks with a timeout (claude-code-book Ch08).
//!
//! For now only the Command hook type is implemented (shell execution).
//! Prompt/Agent/HTTP/Function types are stubbed for the P2 phase.

use std::process::Stdio;
use std::time::Duration;

use tokio::io::AsyncWriteExt;
use tokio::process::Command;
// tokio::time::sleep used via full path in select! below

use super::protocol::{HookDecision, HookInput, HookOutput};
use super::registry::HookConfig;

/// Default 2-second timeout (claude-code-book Ch08 "speculative classifier"
/// uses Promise.race with 2s). Hooks that exceed this are killed and
/// treated as passthrough.
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(2);

/// Run a single Command hook. Returns its decision and output.
pub async fn run_hook(config: &HookConfig, input: &HookInput) -> anyhow::Result<HookOutput> {
    let payload = serde_json::to_vec(input)?;
    let t = Duration::from_millis(config.timeout_ms.max(1));
    let limit = t.min(DEFAULT_TIMEOUT).max(Duration::from_millis(100));

    let mut child = Command::new("sh")
        .arg("-c")
        .arg(&config.command)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    if let Some(mut stdin) = child.stdin.take() {
        // Write payload; ignore broken pipe (hook may exit early).
        let _ = stdin.write_all(&payload).await;
        let _ = stdin.shutdown().await;
    }

    // Take stdout handle before moving child into wait_with_output.
    let stdout_handle = child.stdout.take();
    let result = tokio::time::timeout(limit, child.wait_with_output()).await;
    match result {
        Ok(Ok(out)) => {
            let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if stdout.is_empty() {
                return Ok(HookOutput::default());
            }
            match serde_json::from_str::<HookOutput>(&stdout) {
                Ok(o) => Ok(o),
                Err(_) => {
                    // Non-JSON stdout: treat as passthrough (claude-code-book
                    // allows hooks to just exit 0 for "no opinion").
                    Ok(HookOutput::default())
                }
            }
        }
        Ok(Err(e)) => Err(anyhow::anyhow!("hook process error: {}", e)),
        Err(_) => {
            // Timed out. The timeout future is dropped, which drops
            // wait_with_output's future, which kills the child process.
            // We don't touch child here (it was moved into the future).
            let _ = stdout_handle; // suppress unused
            Ok(HookOutput::default())
        }
    }
}

/// Run a chain of hooks in order. The first non-passthrough decision wins
/// (deny short-circuits; allow stops the chain).
pub async fn run_chain(configs: &[HookConfig], input: &HookInput) -> HookOutput {
    for cfg in configs {
        match run_hook(cfg, input).await {
            Ok(o) => match o.decision() {
                HookDecision::Passthrough => continue,
                _ => return o,
            },
            Err(e) => {
                tracing::warn!("hook {:?} failed: {}", cfg.command, e);
                continue;
            }
        }
    }
    HookOutput::default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::registry::HookConfig;

    #[tokio::test]
    async fn test_hook_allow() {
        let cfg = HookConfig {
            command: r#"echo '{"decision":"allow"}'"#.into(),
            timeout_ms: 1000,
            message: None,
            once: false,
        };
        let input = HookInput {
            event: "PreToolUse".into(),
            tool: Some("Bash".into()),
            input: None,
            session_id: None,
            cwd: None,
        };
        let out = run_hook(&cfg, &input).await.unwrap();
        assert_eq!(out.decision(), HookDecision::Allow);
    }

    #[tokio::test]
    async fn test_hook_deny_with_reason() {
        let cfg = HookConfig {
            command: r#"echo '{"decision":"deny","reason":"forbidden"}'"#.into(),
            timeout_ms: 1000,
            message: None,
            once: false,
        };
        let input = HookInput {
            event: "PreToolUse".into(),
            tool: Some("Bash".into()),
            input: None,
            session_id: None,
            cwd: None,
        };
        let out = run_hook(&cfg, &input).await.unwrap();
        assert_eq!(out.decision(), HookDecision::Deny);
        assert_eq!(out.reason.as_deref(), Some("forbidden"));
    }

    #[tokio::test]
    async fn test_hook_timeout_passthrough() {
        // sleep 5s should timeout at 1s and return passthrough.
        let cfg = HookConfig {
            command: "sleep 5".into(),
            timeout_ms: 500,
            message: None,
            once: false,
        };
        let input = HookInput {
            event: "PreToolUse".into(),
            tool: Some("Bash".into()),
            input: None,
            session_id: None,
            cwd: None,
        };
        let out = run_hook(&cfg, &input).await.unwrap();
        assert_eq!(out.decision(), HookDecision::Passthrough);
    }
}
