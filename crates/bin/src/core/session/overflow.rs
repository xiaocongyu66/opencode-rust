//! Session overflow detection.
//!
//! Ported from `session/overflow.ts`.
//! Determines when a session's context window is full and compaction is needed.

use crate::schema::session::SessionTokens;

/// Buffer reserved for compaction.
pub const COMPACTION_BUFFER: u64 = 20_000;

/// Configuration for compaction.
#[derive(Debug, Clone, Default)]
pub struct CompactionConfig {
    pub auto: Option<bool>,
    pub reserved: Option<u64>,
}

/// Model limits for context window calculations.
#[derive(Debug, Clone, Default)]
pub struct ModelLimits {
    pub context: u64,
    pub input: u64,
    pub output: u64,
}

/// Calculate the usable token budget for a model.
pub fn usable(cfg: &CompactionConfig, model: &ModelLimits, output_token_max: Option<u64>) -> u64 {
    let context = model.context;
    if context == 0 {
        return 0;
    }

    let max_output = output_token_max.unwrap_or(model.output).max(1);
    let reserved = cfg
        .reserved
        .unwrap_or(COMPACTION_BUFFER.min(max_output));

    if model.input > 0 {
        model.input.saturating_sub(reserved)
    } else {
        context.saturating_sub(max_output)
    }
}

/// Check if the session has overflowed its context window.
pub fn is_overflow(
    cfg: &CompactionConfig,
    tokens: &SessionTokens,
    model: &ModelLimits,
    output_token_max: Option<u64>,
) -> bool {
    if cfg.auto == Some(false) {
        return false;
    }
    if model.context == 0 {
        return false;
    }

    let total = if tokens.input + tokens.output + tokens.cache.read + tokens.cache.write > 0.0 {
        tokens.input + tokens.output + tokens.cache.read + tokens.cache.write
    } else {
        0.0
    };

    let budget = usable(cfg, model, output_token_max) as f64;
    total >= budget
}
