//! LLM schema — canonical runtime data model.
//!
//! Migrated from the TypeScript `packages/llm/src/schema/` directory.
//! Covers ids, options, messages, events, and errors.

use std::collections::BTreeMap;
use std::fmt;

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// ids.ts
// ---------------------------------------------------------------------------

pub type ProtocolId = String;
pub type RouteId = String;
pub type ModelId = String;
pub type ProviderId = String;
pub type ResponseId = String;
pub type ContentBlockId = String;
pub type ToolCallId = String;

/// A JSON Schema document. TS models this as `Record<string, unknown>`.
pub type JsonSchema = serde_json::Value;

/// Provider-scoped metadata bag (`{ provider: { key: value } }`).
pub type ProviderMetadata = serde_json::Map<String, serde_json::Value>;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReasoningEffort {
    None,
    Minimal,
    Low,
    Medium,
    High,
    Xhigh,
    Max,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TextVerbosity {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    #[default]
    System,
    User,
    Assistant,
    Tool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FinishReason {
    #[default]
    Unknown,
    Stop,
    Length,
    ToolCalls,
    ContentFilter,
    Error,
}

// ---------------------------------------------------------------------------
// options.ts
// ---------------------------------------------------------------------------

/// `Record<string, Record<string, unknown>>` — provider-scoped options.
pub type ProviderOptions = serde_json::Map<String, serde_json::Value>;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HttpOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<JsonSchema>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<BTreeMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query: Option<BTreeMap<String, String>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerationOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence_penalty: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Vec<String>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelLimits {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<u64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelDefaults {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limits: Option<ModelLimits>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generation: Option<GenerationOptions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider_options: Option<ProviderOptions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub http: Option<HttpOptions>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ModelToolSchemaCompatibility {
    Gemini,
    Moonshot,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelCompatibility {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_schema: Option<ModelToolSchemaCompatibility>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Model {
    pub id: ModelId,
    pub provider: ProviderId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defaults: Option<ModelDefaults>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compatibility: Option<ModelCompatibility>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CacheHintType {
    Ephemeral,
    Persistent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CacheHint {
    pub r#type: CacheHintType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ttl_seconds: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CachePolicyMessageStrategy {
    LatestUserMessage,
    LatestAssistant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CachePolicyMessages {
    Strategy(CachePolicyMessageStrategy),
    Tail { tail: u64 },
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CachePolicyObject {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub messages: Option<CachePolicyMessages>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ttl_seconds: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CachePolicy {
    Mode(CachePolicyMode),
    Object(CachePolicyObject),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CachePolicyMode {
    Auto,
    None,
}

// ---------------------------------------------------------------------------
// messages.ts
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemPart {
    #[serde(rename = "type")]
    pub part_type: String,
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache: Option<CacheHint>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Map<String, serde_json::Value>>,
}

impl SystemPart {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            part_type: "text".to_string(),
            text: text.into(),
            cache: None,
            metadata: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextPart {
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache: Option<CacheHint>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Map<String, serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider_metadata: Option<ProviderMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MediaData {
    Text(String),
    Bytes(Vec<u8>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaPart {
    pub media_type: String,
    pub data: MediaData,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filename: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Map<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ToolContent {
    Text { text: String },
    File {
        uri: String,
        mime: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ToolResultValue {
    Json { value: serde_json::Value },
    Text { value: serde_json::Value },
    Error { value: serde_json::Value },
    Content { value: Vec<ToolContent> },
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolOutput {
    pub structured: serde_json::Value,
    pub content: Vec<ToolContent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolCallPart {
    pub id: String,
    pub name: String,
    pub input: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider_executed: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Map<String, serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider_metadata: Option<ProviderMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolResultPart {
    pub id: String,
    pub name: String,
    pub result: ToolResultValue,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider_executed: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache: Option<CacheHint>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Map<String, serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider_metadata: Option<ProviderMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReasoningPart {
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encrypted: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Map<String, serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider_metadata: Option<ProviderMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum ContentPart {
    Text(TextPart),
    Media(MediaPart),
    ToolCall(ToolCallPart),
    ToolResult(ToolResultPart),
    Reasoning(ReasoningPart),
}

impl ContentPart {
    pub fn text(value: impl Into<String>) -> Self {
        ContentPart::Text(TextPart {
            text: value.into(),
            cache: None,
            metadata: None,
            provider_metadata: None,
        })
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub role: MessageRole,
    pub content: Vec<ContentPart>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Map<String, serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub native: Option<serde_json::Map<String, serde_json::Value>>,
}

impl Message {
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            id: None,
            role: MessageRole::User,
            content: vec![ContentPart::text(content)],
            metadata: None,
            native: None,
        }
    }

    pub fn assistant(content: Vec<ContentPart>) -> Self {
        Self {
            id: None,
            role: MessageRole::Assistant,
            content,
            metadata: None,
            native: None,
        }
    }

    pub fn system(content: impl Into<String>) -> Self {
        Self {
            id: None,
            role: MessageRole::System,
            content: vec![ContentPart::text(content)],
            metadata: None,
            native: None,
        }
    }

    pub fn tool(result: ToolResultPart) -> Self {
        Self {
            id: None,
            role: MessageRole::Tool,
            content: vec![ContentPart::ToolResult(result)],
            metadata: None,
            native: None,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub input_schema: JsonSchema,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_schema: Option<JsonSchema>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache: Option<CacheHint>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Map<String, serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub native: Option<serde_json::Map<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ToolChoiceType {
    Auto,
    None,
    Required,
    Tool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolChoice {
    pub r#type: ToolChoiceType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

impl ToolChoice {
    pub fn named(name: impl Into<String>) -> Self {
        Self {
            r#type: ToolChoiceType::Tool,
            name: Some(name.into()),
        }
    }

    pub fn auto() -> Self {
        Self {
            r#type: ToolChoiceType::Auto,
            name: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ResponseFormat {
    Text,
    Json { schema: JsonSchema },
    Tool { tool: ToolDefinition },
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LlmRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub model: Model,
    pub system: Vec<SystemPart>,
    pub messages: Vec<Message>,
    pub tools: Vec<ToolDefinition>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ToolChoice>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generation: Option<GenerationOptions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider_options: Option<ProviderOptions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub http: Option<HttpOptions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<ResponseFormat>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache: Option<CachePolicy>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Map<String, serde_json::Value>>,
}

impl LlmRequest {
    pub fn update(mut self, patch: LlmRequestPatch) -> Self {
        if let Some(model) = patch.model {
            self.model = model;
        }
        if let Some(system) = patch.system {
            self.system = system;
        }
        if let Some(messages) = patch.messages {
            self.messages = messages;
        }
        if let Some(tools) = patch.tools {
            self.tools = tools;
        }
        if let Some(tool_choice) = patch.tool_choice {
            self.tool_choice = Some(tool_choice);
        }
        if let Some(generation) = patch.generation {
            self.generation = Some(generation);
        }
        if let Some(provider_options) = patch.provider_options {
            self.provider_options = Some(provider_options);
        }
        if let Some(http) = patch.http {
            self.http = Some(http);
        }
        if let Some(response_format) = patch.response_format {
            self.response_format = Some(response_format);
        }
        if let Some(cache) = patch.cache {
            self.cache = Some(cache);
        }
        if let Some(metadata) = patch.metadata {
            self.metadata = Some(metadata);
        }
        if let Some(id) = patch.id {
            self.id = Some(id);
        }
        self
    }
}

#[derive(Debug, Clone, Default)]
pub struct LlmRequestPatch {
    pub id: Option<String>,
    pub model: Option<Model>,
    pub system: Option<Vec<SystemPart>>,
    pub messages: Option<Vec<Message>>,
    pub tools: Option<Vec<ToolDefinition>>,
    pub tool_choice: Option<ToolChoice>,
    pub generation: Option<GenerationOptions>,
    pub provider_options: Option<ProviderOptions>,
    pub http: Option<HttpOptions>,
    pub response_format: Option<ResponseFormat>,
    pub cache: Option<CachePolicy>,
    pub metadata: Option<serde_json::Map<String, serde_json::Value>>,
}

// ---------------------------------------------------------------------------
// events.ts
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Usage {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_tokens: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_tokens: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub non_cached_input_tokens: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_read_input_tokens: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_write_input_tokens: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_tokens: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_tokens: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider_metadata: Option<ProviderMetadata>,
}

impl Usage {
    /// Visible output tokens — `output_tokens` minus `reasoning_tokens`,
    /// clamped to zero.
    pub fn visible_output_tokens(&self) -> u64 {
        let out = self.output_tokens.unwrap_or(0);
        let reasoning = self.reasoning_tokens.unwrap_or(0);
        out.saturating_sub(reasoning)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum LlmEvent {
    StepStart {
        index: u64,
    },
    TextStart {
        id: ContentBlockId,
        #[serde(skip_serializing_if = "Option::is_none")]
        provider_metadata: Option<ProviderMetadata>,
    },
    TextDelta {
        id: ContentBlockId,
        text: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        provider_metadata: Option<ProviderMetadata>,
    },
    TextEnd {
        id: ContentBlockId,
        #[serde(skip_serializing_if = "Option::is_none")]
        provider_metadata: Option<ProviderMetadata>,
    },
    ReasoningStart {
        id: ContentBlockId,
        #[serde(skip_serializing_if = "Option::is_none")]
        provider_metadata: Option<ProviderMetadata>,
    },
    ReasoningDelta {
        id: ContentBlockId,
        text: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        provider_metadata: Option<ProviderMetadata>,
    },
    ReasoningEnd {
        id: ContentBlockId,
        #[serde(skip_serializing_if = "Option::is_none")]
        provider_metadata: Option<ProviderMetadata>,
    },
    ToolInputStart {
        id: ToolCallId,
        name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        provider_metadata: Option<ProviderMetadata>,
    },
    ToolInputDelta {
        id: ToolCallId,
        name: String,
        text: String,
    },
    ToolInputEnd {
        id: ToolCallId,
        name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        provider_metadata: Option<ProviderMetadata>,
    },
    ToolCall {
        id: ToolCallId,
        name: String,
        input: serde_json::Value,
        #[serde(skip_serializing_if = "Option::is_none")]
        provider_executed: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        provider_metadata: Option<ProviderMetadata>,
    },
    ToolResult {
        id: ToolCallId,
        name: String,
        result: ToolResultValue,
        #[serde(skip_serializing_if = "Option::is_none")]
        output: Option<ToolOutput>,
        #[serde(skip_serializing_if = "Option::is_none")]
        provider_executed: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        provider_metadata: Option<ProviderMetadata>,
    },
    ToolError {
        id: ToolCallId,
        name: String,
        message: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        error: Option<serde_json::Value>,
        #[serde(skip_serializing_if = "Option::is_none")]
        provider_metadata: Option<ProviderMetadata>,
    },
    StepFinish {
        index: u64,
        reason: FinishReason,
        #[serde(skip_serializing_if = "Option::is_none")]
        usage: Option<Usage>,
        #[serde(skip_serializing_if = "Option::is_none")]
        provider_metadata: Option<ProviderMetadata>,
    },
    Finish {
        reason: FinishReason,
        #[serde(skip_serializing_if = "Option::is_none")]
        usage: Option<Usage>,
        #[serde(skip_serializing_if = "Option::is_none")]
        provider_metadata: Option<ProviderMetadata>,
    },
    ProviderError {
        message: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        classification: Option<ProviderFailureClassification>,
        #[serde(skip_serializing_if = "Option::is_none")]
        retryable: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        provider_metadata: Option<ProviderMetadata>,
    },
}

impl LlmEvent {
    pub fn text_delta(id: impl Into<String>, text: impl Into<String>) -> Self {
        LlmEvent::TextDelta {
            id: id.into(),
            text: text.into(),
            provider_metadata: None,
        }
    }

    pub fn tool_call(id: impl Into<String>, name: impl Into<String>, input: serde_json::Value) -> Self {
        LlmEvent::ToolCall {
            id: id.into(),
            name: name.into(),
            input,
            provider_executed: None,
            provider_metadata: None,
        }
    }

    pub fn finish(reason: FinishReason) -> Self {
        LlmEvent::Finish {
            reason,
            usage: None,
            provider_metadata: None,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreparedRequest {
    pub id: String,
    pub route: RouteId,
    pub protocol: ProtocolId,
    pub model: Model,
    pub body: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Map<String, serde_json::Value>>,
}

// ---------------------------------------------------------------------------
// Response assembly state machine (from events.ts)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default)]
struct ContentAssembly {
    content_index: usize,
    text: String,
    provider_metadata: Option<ProviderMetadata>,
}

#[derive(Debug, Clone, Default)]
struct ToolInputAssembly {
    name: String,
    text: String,
    provider_metadata: Option<ProviderMetadata>,
}

#[derive(Debug, Clone, Default)]
pub struct ResponseState {
    pub events: Vec<LlmEvent>,
    pub message: Message,
    pub usage: Option<Usage>,
    pub finish_reason: Option<FinishReason>,
    text_parts: BTreeMap<String, ContentAssembly>,
    reasoning_parts: BTreeMap<String, ContentAssembly>,
    tool_inputs: BTreeMap<String, ToolInputAssembly>,
}

impl ResponseState {
    fn empty() -> Self {
        Self {
            events: Vec::new(),
            message: Message::assistant(Vec::new()),
            usage: None,
            finish_reason: None,
            text_parts: BTreeMap::new(),
            reasoning_parts: BTreeMap::new(),
            tool_inputs: BTreeMap::new(),
        }
    }

    fn append_event(&mut self, event: LlmEvent) {
        match &event {
            LlmEvent::Finish { reason, usage, .. } => {
                if let Some(u) = usage {
                    self.usage = Some(u.clone());
                }
                self.finish_reason = Some(reason.clone());
            }
            LlmEvent::ProviderError { .. } => {
                if self.finish_reason.is_none() {
                    self.finish_reason = Some(FinishReason::Error);
                }
            }
            LlmEvent::StepFinish { usage, .. } => {
                if let Some(u) = usage {
                    self.usage = Some(u.clone());
                }
            }
            _ => {}
        }
        self.events.push(event);
    }

    fn ensure_text(&mut self, id: &str, provider_metadata: Option<ProviderMetadata>) {
        if self.text_parts.contains_key(id) {
            return;
        }
        let content_index = self.message.content.len();
        self.message.content.push(ContentPart::Text(TextPart {
            text: String::new(),
            cache: None,
            metadata: None,
            provider_metadata: provider_metadata.clone(),
        }));
        self.text_parts.insert(
            id.to_string(),
            ContentAssembly {
                content_index,
                text: String::new(),
                provider_metadata,
            },
        );
    }

    fn reduce_text_delta(&mut self, id: &str, text: &str, provider_metadata: Option<ProviderMetadata>) {
        let entry = match self.text_parts.get_mut(id) {
            Some(e) => e,
            None => return,
        };
        entry.text.push_str(text);
        if provider_metadata.is_some() {
            entry.provider_metadata = provider_metadata;
        }
        let idx = entry.content_index;
        let new_text = entry.text.clone();
        let pm = entry.provider_metadata.clone();
        if let Some(ContentPart::Text(tp)) = self.message.content.get_mut(idx) {
            tp.text = new_text;
            tp.provider_metadata = pm;
        }
    }

    fn ensure_reasoning(&mut self, id: &str, provider_metadata: Option<ProviderMetadata>) {
        if self.reasoning_parts.contains_key(id) {
            return;
        }
        let content_index = self.message.content.len();
        self.message
            .content
            .push(ContentPart::Reasoning(ReasoningPart {
                text: String::new(),
                encrypted: None,
                metadata: None,
                provider_metadata: provider_metadata.clone(),
            }));
        self.reasoning_parts.insert(
            id.to_string(),
            ContentAssembly {
                content_index,
                text: String::new(),
                provider_metadata,
            },
        );
    }

    fn reduce_reasoning_delta(
        &mut self,
        id: &str,
        text: &str,
        provider_metadata: Option<ProviderMetadata>,
    ) {
        let entry = match self.reasoning_parts.get_mut(id) {
            Some(e) => e,
            None => return,
        };
        entry.text.push_str(text);
        if provider_metadata.is_some() {
            entry.provider_metadata = provider_metadata;
        }
        let idx = entry.content_index;
        let new_text = entry.text.clone();
        let pm = entry.provider_metadata.clone();
        if let Some(ContentPart::Reasoning(rp)) = self.message.content.get_mut(idx) {
            rp.text = new_text;
            rp.provider_metadata = pm;
        }
    }

    fn append_content(&mut self, part: ContentPart) {
        self.message.content.push(part);
    }

    fn reduce(&mut self, event: LlmEvent) {
        match &event {
            LlmEvent::StepStart { .. } => {}
            LlmEvent::TextStart { id, provider_metadata } => {
                self.append_event(event.clone());
                self.ensure_text(id, provider_metadata.clone());
                return;
            }
            LlmEvent::TextDelta { id, text, provider_metadata } => {
                self.append_event(event.clone());
                self.ensure_text(id, provider_metadata.clone());
                self.reduce_text_delta(id, text, provider_metadata.clone());
                return;
            }
            LlmEvent::TextEnd { id, provider_metadata } => {
                self.append_event(event.clone());
                if let Some(entry) = self.text_parts.get_mut(id) {
                    if provider_metadata.is_some() {
                        entry.provider_metadata = provider_metadata.clone();
                    }
                    let idx = entry.content_index;
                    let pm = entry.provider_metadata.clone();
                    if let Some(ContentPart::Text(tp)) = self.message.content.get_mut(idx) {
                        tp.provider_metadata = pm;
                    }
                }
                return;
            }
            LlmEvent::ReasoningStart { id, provider_metadata } => {
                self.append_event(event.clone());
                self.ensure_reasoning(id, provider_metadata.clone());
                return;
            }
            LlmEvent::ReasoningDelta { id, text, provider_metadata } => {
                self.append_event(event.clone());
                self.ensure_reasoning(id, provider_metadata.clone());
                self.reduce_reasoning_delta(id, text, provider_metadata.clone());
                return;
            }
            LlmEvent::ReasoningEnd { id, provider_metadata } => {
                self.append_event(event.clone());
                if let Some(entry) = self.reasoning_parts.get_mut(id) {
                    if provider_metadata.is_some() {
                        entry.provider_metadata = provider_metadata.clone();
                    }
                    let idx = entry.content_index;
                    let pm = entry.provider_metadata.clone();
                    if let Some(ContentPart::Reasoning(rp)) = self.message.content.get_mut(idx) {
                        rp.provider_metadata = pm;
                    }
                }
                return;
            }
            LlmEvent::ToolInputStart { id, name, provider_metadata } => {
                self.append_event(event.clone());
                self.tool_inputs.insert(
                    id.clone(),
                    ToolInputAssembly {
                        name: name.clone(),
                        text: String::new(),
                        provider_metadata: provider_metadata.clone(),
                    },
                );
                return;
            }
            LlmEvent::ToolInputDelta { id, name, text } => {
                self.append_event(event.clone());
                let entry = self.tool_inputs.entry(id.clone()).or_insert_with(|| ToolInputAssembly {
                    name: name.clone(),
                    text: String::new(),
                    provider_metadata: None,
                });
                entry.text.push_str(text);
                return;
            }
            LlmEvent::ToolInputEnd { id, name, provider_metadata } => {
                self.append_event(event.clone());
                let entry = self.tool_inputs.entry(id.clone()).or_insert_with(|| ToolInputAssembly {
                    name: name.clone(),
                    text: String::new(),
                    provider_metadata: None,
                });
                entry.name = name.clone();
                if provider_metadata.is_some() {
                    entry.provider_metadata = provider_metadata.clone();
                }
                return;
            }
            LlmEvent::ToolCall { id, name, input, provider_executed, provider_metadata } => {
                self.tool_inputs.remove(id);
                self.append_content(ContentPart::ToolCall(ToolCallPart {
                    id: id.clone(),
                    name: name.clone(),
                    input: input.clone(),
                    provider_executed: provider_executed.clone(),
                    metadata: None,
                    provider_metadata: provider_metadata.clone(),
                }));
                self.append_event(event.clone());
                return;
            }
            LlmEvent::ToolResult { id, name, result, output, provider_executed, provider_metadata } => {
                self.append_content(ContentPart::ToolResult(ToolResultPart {
                    id: id.clone(),
                    name: name.clone(),
                    result: result.clone(),
                    provider_executed: provider_executed.clone(),
                    cache: None,
                    metadata: None,
                    provider_metadata: provider_metadata.clone(),
                }));
                let _ = output;
                self.append_event(event.clone());
                return;
            }
            LlmEvent::ToolError { .. } | LlmEvent::StepFinish { .. } | LlmEvent::Finish { .. } | LlmEvent::ProviderError { .. } => {
                self.append_event(event.clone());
                return;
            }
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LlmResponse {
    pub message: Message,
    pub events: Vec<LlmEvent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<Usage>,
    pub finish_reason: FinishReason,
}

impl LlmResponse {
    /// Concatenated assistant text assembled from streamed `text-delta` events.
    pub fn text(&self) -> String {
        self.events
            .iter()
            .filter_map(|e| match e {
                LlmEvent::TextDelta { text, .. } => Some(text.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("")
    }

    /// Concatenated reasoning text assembled from streamed `reasoning-delta` events.
    pub fn reasoning(&self) -> String {
        self.events
            .iter()
            .filter_map(|e| match e {
                LlmEvent::ReasoningDelta { text, .. } => Some(text.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("")
    }

    /// Completed tool calls emitted by the provider.
    pub fn tool_calls(&self) -> Vec<&LlmEvent> {
        self.events
            .iter()
            .filter(|e| matches!(e, LlmEvent::ToolCall { .. }))
            .collect()
    }

    /// Purely fold one provider-neutral event into the attempt assembly state.
    pub fn reduce(state: &mut ResponseState, event: LlmEvent) {
        state.reduce(event);
    }

    /// Initial reducer state for assembling one provider attempt.
    pub fn empty_state() -> ResponseState {
        ResponseState::empty()
    }

    /// Return a completed response only after a terminal finish or provider error.
    pub fn complete(state: &ResponseState) -> Option<LlmResponse> {
        let reason = state.finish_reason.clone()?;
        Some(LlmResponse {
            message: state.message.clone(),
            events: state.events.clone(),
            usage: state.usage.clone(),
            finish_reason: reason,
        })
    }

    /// Convenience reducer for callers that already have a collected event list.
    pub fn from_events(events: impl IntoIterator<Item = LlmEvent>) -> Option<LlmResponse> {
        let mut state = ResponseState::empty();
        for event in events {
            state.reduce(event);
        }
        Self::complete(&state)
    }
}

// ---------------------------------------------------------------------------
// errors.ts
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProviderFailureClassification {
    ContextOverflow,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HttpRequestDetails {
    pub method: String,
    pub url: String,
    pub headers: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HttpResponseDetails {
    pub status: u16,
    pub headers: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HttpRateLimitDetails {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry_after_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<BTreeMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remaining: Option<BTreeMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reset: Option<BTreeMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HttpContext {
    pub request: HttpRequestDetails,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response: Option<HttpResponseDetails>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body_truncated: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rate_limit: Option<HttpRateLimitDetails>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AuthenticationKind {
    Missing,
    Invalid,
    Expired,
    InsufficientPermissions,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "_tag")]
pub enum LlmErrorReason {
    InvalidRequest {
        message: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        parameter: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        classification: Option<ProviderFailureClassification>,
        #[serde(skip_serializing_if = "Option::is_none")]
        provider_metadata: Option<ProviderMetadata>,
        #[serde(skip_serializing_if = "Option::is_none")]
        http: Option<HttpContext>,
    },
    NoRoute {
        route: RouteId,
        provider: ProviderId,
        model: ModelId,
    },
    Authentication {
        message: String,
        kind: AuthenticationKind,
        #[serde(skip_serializing_if = "Option::is_none")]
        provider_metadata: Option<ProviderMetadata>,
        #[serde(skip_serializing_if = "Option::is_none")]
        http: Option<HttpContext>,
    },
    RateLimit {
        message: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        retry_after_ms: Option<u64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        rate_limit: Option<HttpRateLimitDetails>,
        #[serde(skip_serializing_if = "Option::is_none")]
        provider_metadata: Option<ProviderMetadata>,
        #[serde(skip_serializing_if = "Option::is_none")]
        http: Option<HttpContext>,
    },
    QuotaExceeded {
        message: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        provider_metadata: Option<ProviderMetadata>,
        #[serde(skip_serializing_if = "Option::is_none")]
        http: Option<HttpContext>,
    },
    ContentPolicy {
        message: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        provider_metadata: Option<ProviderMetadata>,
        #[serde(skip_serializing_if = "Option::is_none")]
        http: Option<HttpContext>,
    },
    ProviderInternal {
        message: String,
        status: u16,
        #[serde(skip_serializing_if = "Option::is_none")]
        retry_after_ms: Option<u64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        provider_metadata: Option<ProviderMetadata>,
        #[serde(skip_serializing_if = "Option::is_none")]
        http: Option<HttpContext>,
    },
    Transport {
        message: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        kind: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        url: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        http: Option<HttpContext>,
    },
    InvalidProviderOutput {
        message: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        route: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        raw: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        provider_metadata: Option<ProviderMetadata>,
    },
    UnknownProvider {
        message: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        status: Option<u16>,
        #[serde(skip_serializing_if = "Option::is_none")]
        provider_metadata: Option<ProviderMetadata>,
        #[serde(skip_serializing_if = "Option::is_none")]
        http: Option<HttpContext>,
    },
}

impl LlmErrorReason {
    pub fn message(&self) -> String {
        match self {
            LlmErrorReason::InvalidRequest { message, .. }
            | LlmErrorReason::Authentication { message, .. }
            | LlmErrorReason::RateLimit { message, .. }
            | LlmErrorReason::QuotaExceeded { message, .. }
            | LlmErrorReason::ContentPolicy { message, .. }
            | LlmErrorReason::ProviderInternal { message, .. }
            | LlmErrorReason::Transport { message, .. }
            | LlmErrorReason::InvalidProviderOutput { message, .. }
            | LlmErrorReason::UnknownProvider { message, .. } => message.clone(),
            LlmErrorReason::NoRoute { route, provider, model } => {
                format!("No LLM route for {provider}/{model} using {route}")
            }
        }
    }

    pub fn retryable(&self) -> bool {
        matches!(
            self,
            LlmErrorReason::RateLimit { .. } | LlmErrorReason::ProviderInternal { .. }
        )
    }

    pub fn retry_after_ms(&self) -> Option<u64> {
        match self {
            LlmErrorReason::RateLimit { retry_after_ms, .. }
            | LlmErrorReason::ProviderInternal { retry_after_ms, .. } => *retry_after_ms,
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LlmError {
    pub module: String,
    pub method: String,
    pub reason: LlmErrorReason,
}

impl LlmError {
    pub fn new(module: impl Into<String>, method: impl Into<String>, reason: LlmErrorReason) -> Self {
        Self {
            module: module.into(),
            method: method.into(),
            reason,
        }
    }

    pub fn network(message: impl Into<String>) -> Self {
        Self::new("LLM", "network", LlmErrorReason::Transport {
            message: message.into(),
            kind: None,
            url: None,
            http: None,
        })
    }

    pub fn provider(message: impl Into<String>) -> Self {
        Self::new("LLM", "provider", LlmErrorReason::ProviderInternal {
            message: message.into(),
            status: 500,
            retry_after_ms: None,
            provider_metadata: None,
            http: None,
        })
    }

    pub fn parse(message: impl Into<String>) -> Self {
        Self::new("LLM", "parse", LlmErrorReason::InvalidRequest {
            message: message.into(),
            parameter: None,
            classification: None,
            provider_metadata: None,
            http: None,
        })
    }

    pub fn retryable(&self) -> bool {
        self.reason.retryable()
    }

    pub fn retry_after_ms(&self) -> Option<u64> {
        self.reason.retry_after_ms()
    }
}

impl fmt::Display for LlmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}: {}", self.module, self.method, self.reason.message())
    }
}

impl std::error::Error for LlmError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolFailure {
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Map<String, serde_json::Value>>,
}

impl ToolFailure {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            error: None,
            metadata: None,
        }
    }
}

impl fmt::Display for ToolFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ToolFailure {}

// ---------------------------------------------------------------------------
// Merge helpers (from options.ts)
// ---------------------------------------------------------------------------

pub fn merge_provider_options(
    items: &[Option<&ProviderOptions>],
) -> Option<ProviderOptions> {
    let mut result: ProviderOptions = serde_json::Map::new();
    for item in items.iter().flatten() {
        for (provider, options) in item.iter() {
            let merged = if let Some(existing) = result.get(provider) {
                merge_json_values(existing, options)
            } else {
                options.clone()
            };
            result.insert(provider.clone(), merged);
        }
    }
    if result.is_empty() {
        None
    } else {
        Some(result)
    }
}

fn merge_json_values(a: &serde_json::Value, b: &serde_json::Value) -> serde_json::Value {
    match (a, b) {
        (serde_json::Value::Object(a), serde_json::Value::Object(b)) => {
            let mut merged = a.clone();
            for (key, value) in b {
                if let Some(existing) = merged.get(key) {
                    merged.insert(key.clone(), merge_json_values(existing, value));
                } else {
                    merged.insert(key.clone(), value.clone());
                }
            }
            serde_json::Value::Object(merged)
        }
        (_, b) => b.clone(),
    }
}

pub fn merge_http_options(items: &[Option<&HttpOptions>]) -> Option<HttpOptions> {
    let mut body: Option<serde_json::Value> = None;
    let mut headers: Option<BTreeMap<String, String>> = None;
    let mut query: Option<BTreeMap<String, String>> = None;

    for item in items.iter().flatten() {
        if let Some(b) = &item.body {
            body = Some(match body.take() {
                Some(existing) => merge_json_values(&existing, b),
                None => b.clone(),
            });
        }
        if let Some(h) = &item.headers {
            let map = headers.get_or_insert_with(BTreeMap::new);
            for (k, v) in h {
                map.insert(k.clone(), v.clone());
            }
        }
        if let Some(q) = &item.query {
            let map = query.get_or_insert_with(BTreeMap::new);
            for (k, v) in q {
                map.insert(k.clone(), v.clone());
            }
        }
    }

    if body.is_none() && headers.is_none() && query.is_none() {
        None
    } else {
        Some(HttpOptions { body, headers, query })
    }
}

pub fn merge_generation_options(items: &[Option<&GenerationOptions>]) -> Option<GenerationOptions> {
    let mut result = GenerationOptions::default();
    let mut has_any = false;
    for item in items.iter().flatten() {
        if let Some(v) = item.max_tokens {
            result.max_tokens = Some(v);
            has_any = true;
        }
        if let Some(v) = item.temperature {
            result.temperature = Some(v);
            has_any = true;
        }
        if let Some(v) = item.top_p {
            result.top_p = Some(v);
            has_any = true;
        }
        if let Some(v) = item.top_k {
            result.top_k = Some(v);
            has_any = true;
        }
        if let Some(v) = item.frequency_penalty {
            result.frequency_penalty = Some(v);
            has_any = true;
        }
        if let Some(v) = item.presence_penalty {
            result.presence_penalty = Some(v);
            has_any = true;
        }
        if let Some(v) = item.seed {
            result.seed = Some(v);
            has_any = true;
        }
        if let Some(v) = &item.stop {
            result.stop = Some(v.clone());
            has_any = true;
        }
    }
    if has_any { Some(result) } else { None }
}
