//! Session processor — handles LLM stream events.
//!
//! Ported from `session/processor.ts`.
//! Processes streaming LLM events: text deltas, tool calls, reasoning,
//! step boundaries, and errors.

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;

use crate::schema::ids::SessionID;
use crate::schema::session::AssistantContent;

/// Doom loop detection threshold.
pub const DOOM_LOOP_THRESHOLD: usize = 3;

/// Processor result.
#[derive(Debug, Clone, PartialEq)]
pub enum ProcessorResult {
    Compact,
    Stop,
    Continue,
}

/// Tool call tracking.
#[derive(Debug, Clone)]
pub struct ToolCall {
    pub part_id: String,
    pub message_id: String,
    pub session_id: SessionID,
    pub done: bool,
}

/// Processor context — holds state during a single LLM stream.
pub struct ProcessorContext {
    pub session_id: SessionID,
    pub tool_calls: HashMap<String, ToolCall>,
    pub should_break: bool,
    pub snapshot: Option<String>,
    pub blocked: bool,
    pub needs_compaction: bool,
    pub current_text: Option<(String, String)>,
    pub reasoning_map: HashMap<String, (String, String)>,
    pub assistant_message_id: String,
}

impl ProcessorContext {
    pub fn new(session_id: SessionID, assistant_message_id: String) -> Self {
        Self {
            session_id,
            tool_calls: HashMap::new(),
            should_break: false,
            snapshot: None,
            blocked: false,
            needs_compaction: false,
            current_text: None,
            reasoning_map: HashMap::new(),
            assistant_message_id,
        }
    }
}

/// LLM stream event types.
#[derive(Debug, Clone)]
pub enum LlmEvent {
    ReasoningStart { id: String, text: String },
    ReasoningDelta { id: String, text: String },
    ReasoningEnd { id: String },
    TextStart { id: String },
    TextDelta { id: String, text: String },
    TextEnd { id: String, text: String },
    ToolInputStart { id: String, name: String },
    ToolInputDelta { id: String, text: String },
    ToolInputEnd { id: String, text: String },
    ToolCall {
        id: String,
        name: String,
        input: serde_json::Value,
    },
    ToolResult {
        id: String,
        result: ToolResult,
    },
    ToolError {
        id: String,
        error: String,
    },
    StepStart { snapshot: Option<String> },
    StepFinish {
        reason: String,
        snapshot: Option<String>,
        usage: Usage,
    },
    ProviderError { message: String },
    Finish,
}

/// Tool result.
#[derive(Debug, Clone)]
pub struct ToolResult {
    pub output: String,
    pub title: String,
    pub metadata: HashMap<String, serde_json::Value>,
    pub attachments: Vec<serde_json::Value>,
}

/// Token usage info.
#[derive(Debug, Clone, Default)]
pub struct Usage {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub reasoning_tokens: u64,
    pub cache_read_tokens: u64,
    pub cache_write_tokens: u64,
    pub total_tokens: u64,
}

/// Session processor — handles LLM events and builds assistant message content.
pub struct SessionProcessor {
    ctx: Arc<RwLock<ProcessorContext>>,
}

impl SessionProcessor {
    pub fn new(session_id: SessionID, assistant_message_id: String) -> Self {
        Self {
            ctx: Arc::new(RwLock::new(ProcessorContext::new(session_id, assistant_message_id))),
        }
    }

    /// Handle a single LLM event.
    pub async fn handle_event(&self, event: &LlmEvent) -> Vec<AssistantContent> {
        let mut ctx = self.ctx.write().await;
        let mut parts = Vec::new();

        match event {
            LlmEvent::ReasoningStart { id, text } => {
                let part_id = crate::core::session::schema::PartID::ascending().to_string();
                ctx.reasoning_map
                    .insert(id.clone(), (part_id.clone(), text.clone()));
            }
            LlmEvent::ReasoningDelta { id, text } => {
                if let Some((_, existing)) = ctx.reasoning_map.get_mut(id) {
                    existing.push_str(text);
                }
            }
            LlmEvent::ReasoningEnd { id } => {
                ctx.reasoning_map.remove(id);
            }
            LlmEvent::TextStart { id } => {
                ctx.current_text = Some((id.clone(), String::new()));
            }
            LlmEvent::TextDelta { text, .. } => {
                if let Some((_, existing)) = ctx.current_text.as_mut() {
                    existing.push_str(text);
                }
            }
            LlmEvent::TextEnd { id, text } => {
                let _ = text;
                if let Some((existing_id, content)) = ctx.current_text.take() {
                    let _ = id;
                    parts.push(AssistantContent::Text {
                        id: existing_id,
                        text: content,
                    });
                }
            }
            LlmEvent::StepStart { snapshot } => {
                if ctx.snapshot.is_none() {
                    ctx.snapshot = snapshot.clone();
                }
            }
            LlmEvent::StepFinish { reason: _, .. } => {
                ctx.snapshot = None;
            }
            LlmEvent::ProviderError { message } => {
                ctx.needs_compaction = message.contains("context");
            }
            _ => {}
        }

        parts
    }

    /// Cleanup pending state after stream ends.
    pub async fn cleanup(&self) -> Vec<AssistantContent> {
        let mut ctx = self.ctx.write().await;
        let mut parts = Vec::new();

        if let Some((id, text)) = ctx.current_text.take() {
            if !text.is_empty() {
                parts.push(AssistantContent::Text { id, text });
            }
        }

        for (_, (id, text)) in ctx.reasoning_map.drain() {
            if !text.is_empty() {
                parts.push(AssistantContent::Reasoning {
                    id,
                    text,
                    provider_metadata: None,
                    time: None,
                });
            }
        }

        parts
    }

    /// Check if compaction is needed.
    pub async fn needs_compaction(&self) -> bool {
        self.ctx.read().await.needs_compaction
    }

    /// Check if processor is blocked.
    pub async fn is_blocked(&self) -> bool {
        self.ctx.read().await.blocked
    }

    /// Get the result of processing.
    pub async fn result(&self) -> ProcessorResult {
        let ctx = self.ctx.read().await;
        if ctx.needs_compaction {
            return ProcessorResult::Compact;
        }
        if ctx.blocked {
            return ProcessorResult::Stop;
        }
        ProcessorResult::Continue
    }
}
