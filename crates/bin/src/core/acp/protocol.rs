//! ACP protocol — Agent Client Protocol event types.
//!
//! Follows claude-code-book Ch02: the agent loop yields five event types
//! that the frontend consumes: stream_request_start, StreamEvent, Message,
//! TombstoneMessage, ToolUseSummaryMessage. This decouples TUI/print/IDE
//! frontends from the specific runner implementation — they all speak ACP.

use serde::{Deserialize, Serialize};

use crate::core::session::runner::RunnerEvent;
use crate::tui::app::{ChatMessage, MessageRole};

/// An ACP event pushed by the agent to all subscribed frontends.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AcpEvent {
    /// A new provider request is starting (turn boundary). Frontends show
    /// a "thinking" indicator.
    StreamRequestStart {
        request_id: String,
        step: usize,
    },

    /// Raw streaming delta from the LLM (text or reasoning). Wrapped from
    /// RunnerEvent::TextDelta / ReasoningDelta. Frontends render incrementally.
    StreamEvent(StreamDelta),

    /// A structured message boundary — new user/assistant/system message
    /// added to the transcript. Frontends add a new bubble.
    Message {
        role: MessageRole,
        text: String,
        queued: bool,
    },

    /// Marks a previously-sent message as retracted (e.g. streaming fallback
    /// rolled back partial output). Frontends hide or annotate the message.
    TombstoneMessage {
        /// Index in the messages list, or message_id when we have one.
        message_index: Option<usize>,
    },

    /// Compact summary of a batch of tool calls (folded in the UI).
    ToolUseSummaryMessage {
        tool_call_ids: Vec<String>,
        summary: String,
    },

    /// Token-budget pressure signal (Ch07 AutoCompact). Frontends show a
    /// toast or warning bar.
    CompactionNeeded {
        tier: crate::core::session::compaction::CompactionTier,
        used: u64,
        effective: u64,
    },

    /// Tool lifecycle events (carried from RunnerEvent).
    ToolStarted {
        tool_name: String,
        call_id: String,
        input: serde_json::Value,
    },
    ToolSuccess {
        tool_name: String,
        call_id: String,
        summary: String,
    },
    ToolFailed {
        tool_name: String,
        call_id: String,
        error: String,
    },

    /// Turn finished — usage stats updated.
    StepFinished {
        step: usize,
        finish_reason: String,
        usage: Option<crate::llm::schema::Usage>,
    },

    /// Terminal: the run is done (success or interrupt).
    Done {
        steps: usize,
        finish_reason: String,
    },

    /// Terminal: an error occurred.
    Error {
        message: String,
    },
}

/// Streaming delta variant — distinguishes text vs reasoning.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StreamDelta {
    /// Assistant text token(s).
    Text { text: String },
    /// Assistant reasoning/thinking token(s).
    Reasoning { text: String },
}

/// Convert a RunnerEvent into an AcpEvent. Returns None for events that
/// don't map to a frontend-visible ACP event (e.g. StepStarted is folded
/// into StreamRequestStart by the bridge).
pub fn from_runner_event(event: RunnerEvent, step: usize) -> Option<AcpEvent> {
    match event {
        RunnerEvent::StepStarted { step } => Some(AcpEvent::StreamRequestStart {
            request_id: format!("turn-{}", step),
            step,
        }),
        RunnerEvent::TextDelta { text } => {
            Some(AcpEvent::StreamEvent(StreamDelta::Text { text }))
        }
        RunnerEvent::TextDone { .. } => None,
        RunnerEvent::ReasoningDelta { text } => {
            Some(AcpEvent::StreamEvent(StreamDelta::Reasoning { text }))
        }
        RunnerEvent::ReasoningDone { .. } => None,
        RunnerEvent::ToolStarted { tool_name, call_id, input } => {
            Some(AcpEvent::ToolStarted { tool_name, call_id, input })
        }
        RunnerEvent::ToolSuccess { tool_name, call_id, summary } => {
            Some(AcpEvent::ToolSuccess { tool_name, call_id, summary })
        }
        RunnerEvent::ToolFailed { tool_name, call_id, error } => {
            Some(AcpEvent::ToolFailed { tool_name, call_id, error })
        }
        RunnerEvent::StepFinished { step, finish_reason, usage } => {
            Some(AcpEvent::StepFinished { step, finish_reason, usage })
        }
        RunnerEvent::CompactionNeeded { tier, used, effective } => {
            Some(AcpEvent::CompactionNeeded { tier, used, effective })
        }
        RunnerEvent::Error { message } => Some(AcpEvent::Error { message }),
        RunnerEvent::Done { result } => Some(AcpEvent::Done {
            steps: result.steps,
            finish_reason: format!("{:?}", result.finish_reason),
        }),
    }
}
