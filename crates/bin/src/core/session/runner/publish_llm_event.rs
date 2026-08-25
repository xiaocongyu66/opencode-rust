//! LLM event publisher — persists one provider turn as durable session events.
//!
//! Ported from `core/src/session/runner/publish-llm-event.ts`.
//! Accumulates streamed LLM events and publishes them as SessionEvents.

use std::collections::HashMap;

use crate::llm::schema::{LlmEvent, ProviderMetadata, Usage};
use crate::schema::ids::{SessionID, SessionMessageID};
use crate::schema::model::ModelRef;
use crate::schema::session_event::*;

fn map_to_provider_metadata(
    src: &Option<ProviderMetadata>,
) -> Option<crate::schema::llm::ProviderMetadata> {
    src.as_ref().map(|m| {
        let mut map = HashMap::new();
        for (k, v) in m {
            if let Some(inner_obj) = v.as_object() {
                let mut inner = HashMap::new();
                for (ik, iv) in inner_obj {
                    inner.insert(ik.clone(), iv.clone());
                }
                map.insert(k.clone(), inner);
            } else {
                let mut inner = HashMap::new();
                inner.insert("value".to_string(), v.clone());
                map.insert(k.clone(), inner);
            }
        }
        map
    })
}

fn json_map_to_hashmap(
    src: serde_json::Map<String, serde_json::Value>,
) -> HashMap<String, serde_json::Value> {
    src.into_iter().collect()
}

fn convert_tool_content(
    content: Vec<crate::llm::schema::ToolContent>,
) -> Vec<crate::schema::llm::ToolContent> {
    content
        .into_iter()
        .map(|c| match c {
            crate::llm::schema::ToolContent::Text { text } => {
                crate::schema::llm::ToolContent::Text { text }
            }
            crate::llm::schema::ToolContent::File { uri, mime, name } => {
                crate::schema::llm::ToolContent::File { uri, mime, name }
            }
        })
        .collect()
}

#[derive(Debug, Clone, Default)]
pub struct StepTokens {
    pub input: u64,
    pub output: u64,
    pub reasoning: u64,
    pub cache_read: u64,
    pub cache_write: u64,
}

#[derive(Debug, Clone, Default)]
pub struct StepSettlement {
    pub finish: String,
    pub tokens: StepTokens,
}

#[derive(Debug, Clone)]
pub struct PublisherInput {
    pub session_id: SessionID,
    pub agent: String,
    pub model: ModelRef,
    pub snapshot: Option<String>,
}

struct ToolTracking {
    assistant_message_id: SessionMessageID,
    name: String,
    input_ended: bool,
    called: bool,
    settled: bool,
    provider_executed: bool,
    provider_metadata: Option<ProviderMetadata>,
}

struct FragmentBuffer {
    chunks: HashMap<String, Vec<String>>,
}

impl FragmentBuffer {
    fn new() -> Self {
        Self { chunks: HashMap::new() }
    }
    fn start(&mut self, id: &str) -> Result<(), String> {
        if self.chunks.contains_key(id) {
            return Err(format!("Duplicate start: {}", id));
        }
        self.chunks.insert(id.to_string(), Vec::new());
        Ok(())
    }
    fn append(&mut self, id: &str, value: &str) -> Result<(), String> {
        match self.chunks.get_mut(id) {
            Some(buf) => {
                buf.push(value.to_string());
                Ok(())
            }
            None => Err(format!("Delta before start: {}", id)),
        }
    }
    fn end(&mut self, id: &str) -> Result<String, String> {
        match self.chunks.remove(id) {
            Some(buf) => Ok(buf.join("")),
            None => Err(format!("End before start: {}", id)),
        }
    }
    fn flush(&mut self) -> Vec<(String, String)> {
        let ids: Vec<String> = self.chunks.keys().cloned().collect();
        let mut results = Vec::new();
        for id in ids {
            if let Ok(text) = self.end(&id) {
                results.push((id, text));
            }
        }
        results
    }
}

pub struct LlmEventPublisher {
    input: PublisherInput,
    tools: HashMap<String, ToolTracking>,
    text_buf: FragmentBuffer,
    reasoning_buf: FragmentBuffer,
    tool_input_buf: FragmentBuffer,
    assistant_message_id: Option<SessionMessageID>,
    assistant_active: bool,
    assistant_failed: bool,
    provider_failed: bool,
    step_settlement: Option<StepSettlement>,
}

impl LlmEventPublisher {
    pub fn new(input: PublisherInput) -> Self {
        Self {
            input,
            tools: HashMap::new(),
            text_buf: FragmentBuffer::new(),
            reasoning_buf: FragmentBuffer::new(),
            tool_input_buf: FragmentBuffer::new(),
            assistant_message_id: None,
            assistant_active: false,
            assistant_failed: false,
            provider_failed: false,
            step_settlement: None,
        }
    }

    pub fn has_active_assistant(&self) -> bool {
        self.assistant_active
    }
    pub fn has_assistant_started(&self) -> bool {
        self.assistant_message_id.is_some()
    }
    pub fn has_provider_error(&self) -> bool {
        self.provider_failed
    }
    pub fn step_settlement(&self) -> Option<&StepSettlement> {
        self.step_settlement.as_ref()
    }

    pub fn start_assistant(&mut self) -> SessionMessageID {
        if let Some(id) = &self.assistant_message_id {
            return id.clone();
        }
        let id = SessionMessageID::new();
        self.assistant_message_id = Some(id.clone());
        self.assistant_active = true;
        id
    }

    pub fn assistant_message_id(&self) -> Option<&SessionMessageID> {
        self.assistant_message_id.as_ref()
    }

    fn current_assistant_message_id(&self) -> Result<&SessionMessageID, String> {
        self.assistant_message_id
            .as_ref()
            .ok_or_else(|| "Tool event before assistant step start".to_string())
    }

    fn tokens(usage: Option<&Usage>) -> StepTokens {
        let u = usage.cloned().unwrap_or_default();
        StepTokens {
            input: u.non_cached_input_tokens.unwrap_or(0),
            output: u.visible_output_tokens(),
            reasoning: u.reasoning_tokens.unwrap_or(0),
            cache_read: u.cache_read_input_tokens.unwrap_or(0),
            cache_write: u.cache_write_input_tokens.unwrap_or(0),
        }
    }

    pub fn flush(&mut self) -> Vec<SessionEvent> {
        let mut events = Vec::new();
        for (id, text) in self.text_buf.flush() {
            if let Some(msg_id) = &self.assistant_message_id {
                events.push(SessionEvent::TextEnded(SessionTextEnded {
                    base: self.base(),
                    assistant_message_id: msg_id.clone(),
                    text_id: id,
                    text,
                }));
            }
        }
        for (id, text) in self.reasoning_buf.flush() {
            if let Some(msg_id) = &self.assistant_message_id {
                events.push(SessionEvent::ReasoningEnded(SessionReasoningEnded {
                    base: self.base(),
                    assistant_message_id: msg_id.clone(),
                    reasoning_id: id,
                    text,
                    provider_metadata: None,
                }));
            }
        }
        for (id, text) in self.tool_input_buf.flush() {
            if let Some(tool) = self.tools.get(&id) {
                events.push(SessionEvent::ToolInputEnded(SessionToolInputEnded {
                    base: self.base(),
                    assistant_message_id: tool.assistant_message_id.clone(),
                    call_id: id,
                    text,
                }));
            }
        }
        events
    }

    fn base(&self) -> SessionBase {
        SessionBase {
            timestamp: chrono::Utc::now(),
            session_id: self.input.session_id.clone(),
        }
    }

    pub fn fail_assistant(&mut self, message: &str) -> Vec<SessionEvent> {
        if self.assistant_failed {
            return vec![];
        }
        let mut events = self.flush();
        let msg_id = self.start_assistant();
        self.assistant_active = false;
        self.assistant_failed = true;
        events.push(SessionEvent::StepFailed(SessionStepFailed {
            base: self.base(),
            assistant_message_id: msg_id,
            error: crate::schema::session::SessionMessageUnknownError {
                error_type: "unknown".to_string(),
                message: message.to_string(),
            },
        }));
        events
    }

    pub fn fail_unsettled_tools(&mut self, message: &str, hosted_only: bool) -> Vec<SessionEvent> {
        let mut events = Vec::new();
        let base = self.base();
        for (call_id, tool) in &mut self.tools {
            if tool.settled || (hosted_only && !tool.provider_executed) {
                continue;
            }
            tool.settled = true;
            let assistant_message_id = tool.assistant_message_id.clone();
            let provider_executed_val = tool.provider_executed;
            let provider_metadata_val = tool.provider_metadata.clone();
            events.push(SessionEvent::ToolFailed(SessionToolFailed {
                base: base.clone(),
                assistant_message_id,
                call_id: call_id.clone(),
                error: crate::schema::session::SessionMessageUnknownError {
                    error_type: "unknown".to_string(),
                    message: message.to_string(),
                },
                result: None,
                provider: SessionToolProvider {
                    executed: provider_executed_val,
                    metadata: map_to_provider_metadata(&provider_metadata_val),
                },
            }));
        }
        events
    }

    pub fn publish(&mut self, event: &LlmEvent) -> Vec<SessionEvent> {
        let mut events = Vec::new();
        match event {
            LlmEvent::StepStart { .. } => {}
            LlmEvent::TextStart { id, .. } => {
                let _ = self.text_buf.start(id);
                let msg_id = self.start_assistant();
                events.push(SessionEvent::TextStarted(SessionTextStarted {
                    base: self.base(),
                    assistant_message_id: msg_id,
                    text_id: id.clone(),
                }));
            }
            LlmEvent::TextDelta { id, text, .. } => {
                let _ = self.text_buf.append(id, text);
                if let Ok(msg_id) = self.current_assistant_message_id() {
                    events.push(SessionEvent::TextDelta(SessionTextDelta {
                        base: self.base(),
                        assistant_message_id: msg_id.clone(),
                        text_id: id.clone(),
                        delta: text.clone(),
                    }));
                }
            }
            LlmEvent::TextEnd { id, .. } => {
                if let Ok(text) = self.text_buf.end(id) {
                    if let Ok(msg_id) = self.current_assistant_message_id() {
                        events.push(SessionEvent::TextEnded(SessionTextEnded {
                            base: self.base(),
                            assistant_message_id: msg_id.clone(),
                            text_id: id.clone(),
                            text,
                        }));
                    }
                }
            }
            LlmEvent::ReasoningStart { id, provider_metadata, .. } => {
                let _ = self.reasoning_buf.start(id);
                let msg_id = self.start_assistant();
                events.push(SessionEvent::ReasoningStarted(SessionReasoningStarted {
                    base: self.base(),
                    assistant_message_id: msg_id,
                    reasoning_id: id.clone(),
                    provider_metadata: map_to_provider_metadata(&provider_metadata.clone()),
                }));
            }
            LlmEvent::ReasoningDelta { id, text, .. } => {
                let _ = self.reasoning_buf.append(id, text);
                if let Ok(msg_id) = self.current_assistant_message_id() {
                    events.push(SessionEvent::ReasoningDelta(SessionReasoningDelta {
                        base: self.base(),
                        assistant_message_id: msg_id.clone(),
                        reasoning_id: id.clone(),
                        delta: text.clone(),
                    }));
                }
            }
            LlmEvent::ReasoningEnd { id, provider_metadata, .. } => {
                if let Ok(text) = self.reasoning_buf.end(id) {
                    if let Ok(msg_id) = self.current_assistant_message_id() {
                        events.push(SessionEvent::ReasoningEnded(SessionReasoningEnded {
                            base: self.base(),
                            assistant_message_id: msg_id.clone(),
                            reasoning_id: id.clone(),
                            text,
                            provider_metadata: map_to_provider_metadata(&provider_metadata.clone()),
                        }));
                    }
                }
            }
            LlmEvent::ToolInputStart { id, name, .. } => {
                if self.tools.contains_key(id) {
                    return events;
                }
                let msg_id = self.start_assistant();
                self.tools.insert(
                    id.clone(),
                    ToolTracking {
                        assistant_message_id: msg_id.clone(),
                        name: name.clone(),
                        input_ended: false,
                        called: false,
                        settled: false,
                        provider_executed: false,
                        provider_metadata: None,
                    },
                );
                let _ = self.tool_input_buf.start(id);
                events.push(SessionEvent::ToolInputStarted(SessionToolInputStarted {
                    base: self.base(),
                    assistant_message_id: msg_id,
                    call_id: id.clone(),
                    name: name.clone(),
                }));
            }
            LlmEvent::ToolInputDelta { id, name, text, .. } => {
                if let Some(tool) = self.tools.get(id) {
                    if tool.name != *name {
                        return events;
                    }
                    if tool.input_ended {
                        return events;
                    }
                }
                let _ = self.tool_input_buf.append(id, text);
                if let Some(tool) = self.tools.get(id) {
                    events.push(SessionEvent::ToolInputDelta(SessionToolInputDelta {
                        base: self.base(),
                        assistant_message_id: tool.assistant_message_id.clone(),
                        call_id: id.clone(),
                        delta: text.clone(),
                    }));
                }
            }
            LlmEvent::ToolInputEnd { id, name, .. } => {
                if let Some(tool) = self.tools.get_mut(id) {
                    if tool.name != *name || tool.input_ended {
                        return events;
                    }
                    tool.input_ended = true;
                }
                if let Ok(text) = self.tool_input_buf.end(id) {
                    if let Some(tool) = self.tools.get(id) {
                        events.push(SessionEvent::ToolInputEnded(SessionToolInputEnded {
                            base: self.base(),
                            assistant_message_id: tool.assistant_message_id.clone(),
                            call_id: id.clone(),
                            text,
                        }));
                    }
                }
            }
            LlmEvent::ToolCall { id, name, input, provider_executed, provider_metadata, .. } => {
                if !self.tools.contains_key(id) {
                    let msg_id = self.start_assistant();
                    self.tools.insert(
                        id.clone(),
                        ToolTracking {
                            assistant_message_id: msg_id,
                            name: name.clone(),
                            input_ended: false,
                            called: false,
                            settled: false,
                            provider_executed: false,
                            provider_metadata: None,
                        },
                    );
                    let _ = self.tool_input_buf.start(id);
                }
                let tool = self.tools.get_mut(id).unwrap();
                if !tool.input_ended {
                    tool.input_ended = true;
                }
                if tool.called {
                    return events;
                }
                tool.called = true;
                tool.provider_executed = provider_executed.unwrap_or(false);
                tool.provider_metadata = provider_metadata.clone();

                let input_record = if input.is_object() {
                    input.clone()
                } else {
                    serde_json::json!({ "value": input })
                };
                let input_map = if let serde_json::Value::Object(m) = input_record {
                    m
                } else {
                    serde_json::Map::new()
                };

                let assistant_message_id = tool.assistant_message_id.clone();
                let provider_executed_val = tool.provider_executed;
                events.push(SessionEvent::ToolCalled(SessionToolCalled {
                    base: self.base(),
                    assistant_message_id,
                    call_id: id.clone(),
                    tool: name.clone(),
                    input: json_map_to_hashmap(input_map),
                    provider: SessionToolProvider {
                        executed: provider_executed_val,
                        metadata: map_to_provider_metadata(&provider_metadata.clone()),
                    },
                }));
            }
            LlmEvent::ToolResult { id, name, result, output, provider_executed, provider_metadata } => {
                if let Some(tool) = self.tools.get_mut(id) {
                    if tool.name != *name {
                        return events;
                    }
                    if tool.settled {
                        return events;
                    }
                    tool.settled = true;
                }

                let (structured, content, error_msg) = match result {
                    crate::llm::schema::ToolResultValue::Error { value } => {
                        let msg = if let Some(s) = value.as_str() {
                            s.to_string()
                        } else {
                            serde_json::to_string(value).unwrap_or_default()
                        };
                        (serde_json::Map::new(), vec![], Some(msg))
                    }
                    _ => {
                        let structured = output
                            .as_ref()
                            .map(|o| &o.structured)
                            .cloned()
                            .unwrap_or(serde_json::Value::Null);
                        let structured_map = if let serde_json::Value::Object(m) = structured {
                            m
                        } else {
                            let mut m = serde_json::Map::new();
                            m.insert("value".to_string(), structured);
                            m
                        };
                        let content = output.as_ref().map(|o| o.content.clone()).unwrap_or_default();
                        (structured_map, content, None)
                    }
                };

                let provider = SessionToolProvider {
                    executed: provider_executed.unwrap_or(false),
                    metadata: map_to_provider_metadata(&provider_metadata.clone()),
                };

                if let Some(msg) = error_msg {
                    if let Some(tool) = self.tools.get(id) {
                        events.push(SessionEvent::ToolFailed(SessionToolFailed {
                            base: self.base(),
                            assistant_message_id: tool.assistant_message_id.clone(),
                            call_id: id.clone(),
                            error: crate::schema::session::SessionMessageUnknownError { error_type: "unknown".to_string(), message: msg },
                            result: Some(serde_json::to_value(result).unwrap_or(serde_json::Value::Null)),
                            provider,
                        }));
                    }
                } else if let Some(tool) = self.tools.get(id) {
                    events.push(SessionEvent::ToolSuccess(SessionToolSuccess {
                        base: self.base(),
                        assistant_message_id: tool.assistant_message_id.clone(),
                        call_id: id.clone(),
                        structured: json_map_to_hashmap(structured),
                        content: convert_tool_content(content),
                        output_paths: None,
                        result: None,
                        provider,
                    }));
                }
            }
            LlmEvent::ToolError { id, name, message, .. } => {
                if let Some(tool) = self.tools.get_mut(id) {
                    if tool.name != *name || tool.settled {
                        return events;
                    }
                    tool.settled = true;
                    let assistant_message_id = tool.assistant_message_id.clone();
                    let provider_executed_val = tool.provider_executed;
                    let provider_metadata_val = tool.provider_metadata.clone();
                    events.push(SessionEvent::ToolFailed(SessionToolFailed {
                        base: self.base(),
                        assistant_message_id,
                        call_id: id.clone(),
                        error: crate::schema::session::SessionMessageUnknownError {
                            error_type: "unknown".to_string(),
                            message: message.clone(),
                        },
                        result: None,
                        provider: SessionToolProvider {
                            executed: provider_executed_val,
                    metadata: map_to_provider_metadata(&provider_metadata_val),
                        },
                    }));
                }
            }
            LlmEvent::StepFinish { reason, usage, .. } => {
                events.extend(self.flush());
                self.assistant_active = false;
                if self.step_settlement.is_some() {
                    return events;
                }
                self.step_settlement = Some(StepSettlement {
                    finish: format!("{:?}", reason),
                    tokens: Self::tokens(usage.as_ref()),
                });
            }
            LlmEvent::Finish { .. } => {}
            LlmEvent::ProviderError { message, .. } => {
                self.provider_failed = true;
                events.extend(self.fail_assistant(message));
            }
        }
        events
    }
}
