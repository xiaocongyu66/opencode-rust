//! Session compaction logic.
//!
//! Ported from `session/compaction.ts`.
//! Manages context window compaction by summarizing old messages
//! and pruning tool outputs.

use std::sync::Arc;

use tokio::sync::RwLock;

use crate::schema::ids::SessionID;
use crate::schema::session::SessionMessage;

/// Minimum tokens to prune.
pub const PRUNE_MINIMUM: u64 = 20_000;
/// Token threshold before pruning begins.
pub const PRUNE_PROTECT: u64 = 40_000;
/// Max characters for tool output during compaction.
pub const TOOL_OUTPUT_MAX_CHARS: usize = 2_000;
/// Tools whose outputs are never pruned.
pub const PRUNE_PROTECTED_TOOLS: &[&str] = &["skill"];
/// Min tokens to preserve in recent context.
pub const MIN_PRESERVE_RECENT_TOKENS: u64 = 2_000;
/// Max tokens to preserve in recent context.
pub const MAX_PRESERVE_RECENT_TOKENS: u64 = 15_000;

/// Compaction event types.
pub const EVENT_COMPACTION_STARTED: &str = "session.compaction.started";
pub const EVENT_COMPACTION_ENDED: &str = "session.compaction.ended";
pub const EVENT_COMPACTED: &str = "session.compacted";

/// Result of a compaction process.
#[derive(Debug, Clone, PartialEq)]
pub enum CompactionResult {
    Continue,
    Stop,
    Compact,
}

/// A turn boundary in the conversation.
#[derive(Debug, Clone)]
pub struct Turn {
    pub start: usize,
    pub end: usize,
    pub id: String,
}

/// A tail reference for compaction.
#[derive(Debug, Clone)]
pub struct Tail {
    pub start: usize,
    pub id: String,
}

/// State for compaction selection.
#[derive(Debug, Clone, Default)]
pub struct SelectionResult {
    pub head: Vec<usize>,
    pub tail_start_id: Option<String>,
}

/// Serialize a message for compaction prompt.
pub fn serialize_message(msg: &SessionMessage) -> String {
    match msg {
        SessionMessage::User { text, .. } => {
            if text.is_empty() {
                String::new()
            } else {
                format!("[User]: {}", text)
            }
        }
        SessionMessage::Assistant { content, .. } => {
            let mut parts = Vec::new();
            for item in content {
                match item {
                    crate::schema::session::AssistantContent::Text { text, .. } => {
                        if !text.is_empty() {
                            parts.push(format!("[Assistant]: {}", text));
                        }
                    }
                    crate::schema::session::AssistantContent::Reasoning { text, .. } => {
                        if !text.is_empty() {
                            parts.push(format!("[Assistant reasoning]: {}", text));
                        }
                    }
                    crate::schema::session::AssistantContent::Tool { name, state, .. } => {
                        let call = format!("[Assistant tool call]: {}", name);
                        parts.push(call);
                        match state {
                            crate::schema::session::ToolState::Completed { .. } => {
                                parts.push("[Tool result]: [completed]".to_string());
                            }
                            crate::schema::session::ToolState::Error { error, .. } => {
                                parts.push(format!("[Tool error]: {}", error.message));
                            }
                            _ => {}
                        }
                    }
                }
            }
            parts.join("\n")
        }
        SessionMessage::Compaction { summary, .. } => {
            format!("[Compaction summary]: {}", summary)
        }
        SessionMessage::Synthetic { text, .. } => {
            format!("[System]: {}", text)
        }
        SessionMessage::System { text, .. } => {
            format!("[System]: {}", text)
        }
        SessionMessage::Shell { command, output, .. } => {
            format!("[Shell]: {}\n{}", command, output)
        }
        _ => String::new(),
    }
}

/// Truncate tool output to max chars.
pub fn truncate_tool_output(value: &str) -> String {
    if value.len() <= TOOL_OUTPUT_MAX_CHARS {
        return value.to_string();
    }
    format!(
        "{}\n[truncated]",
        &value[..TOOL_OUTPUT_MAX_CHARS.min(value.len())]
    )
}

// ============================================================================
// Four-tier compaction (claude-code-book Ch07 design)
// ============================================================================

/// Token usage thresholds as fraction of effective context window.
/// Safe < Warning < AutoCompact < Blocking.
pub const THRESHOLD_SAFE: f64 = 0.85;
pub const THRESHOLD_AUTOCOMPACT: f64 = 0.90;
pub const THRESHOLD_BLOCKING: f64 = 0.95;

/// Reserve for the compaction LLM call itself (it needs output tokens to
/// generate the summary). min(max_output, 20_000) per the book.
pub const COMPACTION_RESERVE: u64 = 20_000;

/// Circuit breaker: after this many consecutive failures, stop trying.
pub const MAX_CONSECUTIVE_FAILURES: u32 = 3;

/// Compaction tier selected by current token usage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum CompactionTier {
    /// 0-85%: no action needed.
    None,
    /// 85-90%: warn user, no compaction yet.
    Warning,
    /// 90-95%: trigger AutoCompact (LLM summary of older history).
    AutoCompact,
    /// 95-100%: block new requests until compaction succeeds.
    Blocking,
}

/// Circuit breaker state machine (Closed → HalfOpen → Open).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BreakerState {
    /// Normal operation, failure counter = 0.
    Closed,
    /// At least one failure; still trying.
    HalfOpen,
    /// MAX_CONSECUTIVE_FAILURES reached; skip further attempts.
    Open,
}

#[derive(Debug, Clone, Copy)]
pub struct CircuitBreaker {
    pub state: BreakerState,
    pub failures: u32,
}

impl Default for CircuitBreaker {
    fn default() -> Self {
        Self { state: BreakerState::Closed, failures: 0 }
    }
}

impl CircuitBreaker {
    /// Record a compaction failure; transitions to Open after threshold.
    pub fn record_failure(&mut self) {
        self.failures = self.failures.saturating_add(1);
        self.state = if self.failures >= MAX_CONSECUTIVE_FAILURES {
            BreakerState::Open
        } else {
            BreakerState::HalfOpen
        };
    }

    /// Record a success; resets to Closed.
    pub fn record_success(&mut self) {
        self.failures = 0;
        self.state = BreakerState::Closed;
    }

    /// Whether compaction should be attempted (false when Open).
    pub fn should_try(&self) -> bool {
        self.state != BreakerState::Open
    }
}

/// Effective context window = model_window - min(max_output, COMPACTION_RESERVE).
/// This is the space actually available for conversation history.
pub fn effective_window(model_window: u64, max_output: u64) -> u64 {
    let reserve = max_output.min(COMPACTION_RESERVE);
    model_window.saturating_sub(reserve)
}

/// Pick the compaction tier based on current token usage.
/// `used` / `effective` ratio determines the tier.
pub fn pick_tier(used: u64, effective: u64) -> CompactionTier {
    if effective == 0 {
        return CompactionTier::None;
    }
    let ratio = used as f64 / effective as f64;
    if ratio >= THRESHOLD_BLOCKING {
        CompactionTier::Blocking
    } else if ratio >= THRESHOLD_AUTOCOMPACT {
        CompactionTier::AutoCompact
    } else if ratio >= THRESHOLD_SAFE {
        CompactionTier::Warning
    } else {
        CompactionTier::None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_effective_window() {
        // 200k window, 16k max output → 200k - 16k = 184k
        assert_eq!(effective_window(200_000, 16_384), 183_616);
        // 200k window, 30k max output → 200k - 20k = 180k (capped at reserve)
        assert_eq!(effective_window(200_000, 30_000), 180_000);
        // tiny window edge case
        assert_eq!(effective_window(10_000, 5_000), 0);
    }

    #[test]
    fn test_pick_tier_thresholds() {
        let eff = 100_000u64;
        assert_eq!(pick_tier(0, eff), CompactionTier::None);
        assert_eq!(pick_tier(84_999, eff), CompactionTier::None);
        assert_eq!(pick_tier(85_000, eff), CompactionTier::Warning);
        assert_eq!(pick_tier(89_999, eff), CompactionTier::Warning);
        assert_eq!(pick_tier(90_000, eff), CompactionTier::AutoCompact);
        assert_eq!(pick_tier(94_999, eff), CompactionTier::AutoCompact);
        assert_eq!(pick_tier(95_000, eff), CompactionTier::Blocking);
        assert_eq!(pick_tier(100_000, eff), CompactionTier::Blocking);
        // edge: zero effective
        assert_eq!(pick_tier(100, 0), CompactionTier::None);
    }

    #[test]
    fn test_circuit_breaker_transitions() {
        let mut b = CircuitBreaker::default();
        assert_eq!(b.state, BreakerState::Closed);
        assert!(b.should_try());

        b.record_failure();
        assert_eq!(b.state, BreakerState::HalfOpen);
        assert!(b.should_try());

        b.record_failure();
        b.record_failure();
        assert_eq!(b.state, BreakerState::Open);
        assert!(!b.should_try());

        // success resets
        b.record_success();
        assert_eq!(b.state, BreakerState::Closed);
        assert_eq!(b.failures, 0);
    }
}

/// Identify turns in a message list.
pub fn turns(messages: &[SessionMessage]) -> Vec<Turn> {
    let mut result = Vec::new();
    for (i, msg) in messages.iter().enumerate() {
        if !matches!(msg, SessionMessage::User { .. }) {
            continue;
        }
        if matches!(msg, SessionMessage::Compaction { .. }) {
            continue;
        }
        result.push(Turn {
            start: i,
            end: messages.len(),
            id: match msg {
                SessionMessage::User { id, .. } => id.to_string(),
                _ => String::new(),
            },
        });
    }
    for i in 0..result.len().saturating_sub(1) {
        result[i].end = result[i + 1].start;
    }
    result
}

/// Session compaction manager.
pub struct SessionCompactionManager {
    state: Arc<RwLock<CompactionState>>,
}

#[derive(Debug, Default)]
pub struct CompactionState {
    pub active: bool,
    pub session_id: Option<SessionID>,
}

impl SessionCompactionManager {
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(CompactionState::default())),
        }
    }

    pub async fn is_active(&self) -> bool {
        self.state.read().await.active
    }

    pub async fn set_active(&self, session_id: Option<SessionID>) {
        let mut state = self.state.write().await;
        state.active = session_id.is_some();
        state.session_id = session_id;
    }
}

impl Default for SessionCompactionManager {
    fn default() -> Self {
        Self::new()
    }
}
