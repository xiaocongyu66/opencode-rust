//! Generic OpenAI-compatible provider.
use async_trait::async_trait;
use crate::llm::openai_api::*;
use crate::llm::provider::LlmProvider;
use crate::llm::schema::{ContentPart, FinishReason, LlmError, LlmRequest, LlmResponse, Message, MessageRole, ToolResultPart, ToolResultValue, Usage};
use futures::StreamExt;

pub struct OpenAICompatibleProvider { api_key: String, base_url: String, client: reqwest::Client }

/// Accumulated state for a tool call being streamed in from the LLM.
/// OpenAI streams tool_calls incrementally: the name arrives in the first
/// chunk, arguments JSON arrives in pieces across subsequent chunks.
#[derive(Debug, Default)]
struct PendingToolCall {
    id: Option<String>,
    name: Option<String>,
    arguments: String,
}

impl OpenAICompatibleProvider {
    pub fn new(base_url: impl Into<String>, api_key: impl Into<String>) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(300))
            .connect_timeout(std::time::Duration::from_secs(30))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        Self { api_key: api_key.into(), base_url: base_url.into(), client }
    }
    pub fn from_env() -> Option<Self> {
        let base_url = std::env::var("OPENAI_BASE_URL").ok()?;
        let api_key = std::env::var("OPENAI_API_KEY").or(std::env::var("LLM_API_KEY")).ok()?;
        Some(Self::new(base_url, api_key))
    }
}
fn convert_request(req: &LlmRequest) -> ChatCompletionRequest {
    // Ported from claude-code-best's openaiConvertMessages.ts.
    // Key conversions:
    // - system prompt → role: "system"
    // - assistant message with tool_use blocks → tool_calls[] on assistant message
    // - tool_result blocks → role: "tool" + tool_call_id
    // - thinking/reasoning blocks → reasoning_content (DeepSeek needs this passed back)
    // - CRITICAL: tool messages must come BEFORE any user message (OpenAI API requirement)
    let mut messages: Vec<ChatCompletionMessage> = Vec::new();

    for m in &req.messages {
        match m.role {
            MessageRole::System => {
                let content: String = m.content.iter().filter_map(|p| match p {
                    ContentPart::Text(t) => Some(t.text.clone()),
                    _ => None,
                }).collect();
                messages.push(ChatCompletionMessage {
                    role: ChatCompletionRole::System,
                    content: if content.is_empty() { None } else { Some(content) },
                    name: None, tool_calls: None, tool_call_id: None,
                });
            }
            MessageRole::User => {
                // Split content into text parts and tool_result parts.
                // Tool results must be emitted as separate role:"tool" messages
                // BEFORE any user text message (OpenAI API requirement).
                let mut text_parts: Vec<String> = Vec::new();
                let mut tool_results: Vec<ToolResultPart> = Vec::new();

                for p in &m.content {
                    match p {
                        ContentPart::Text(t) => text_parts.push(t.text.clone()),
                        ContentPart::ToolResult(tr) => tool_results.push(tr.clone()),
                        _ => {}
                    }
                }

                // Emit tool results first (before user text).
                for tr in &tool_results {
                    let content = match &tr.result {
                        ToolResultValue::Json { value } => value.to_string(),
                        ToolResultValue::Text { value } => value.to_string(),
                        ToolResultValue::Error { value } => value.to_string(),
                        ToolResultValue::Content { value } => {
                            value.iter().map(|c| match c {
                                crate::llm::schema::ToolContent::Text { text } => text.clone(),
                                _ => String::new(),
                            }).collect::<Vec<_>>().join("\n")
                        }
                    };
                    messages.push(ChatCompletionMessage {
                        role: ChatCompletionRole::Tool,
                        content: Some(content),
                        name: None,
                        tool_calls: None,
                        tool_call_id: Some(tr.id.clone()),
                    });
                }

                // Then emit the user text (if any).
                if !text_parts.is_empty() {
                    messages.push(ChatCompletionMessage {
                        role: ChatCompletionRole::User,
                        content: Some(text_parts.join("\n")),
                        name: None, tool_calls: None, tool_call_id: None,
                    });
                }
            }
            MessageRole::Assistant => {
                // Split into text parts, tool_calls, and reasoning.
                let mut text_parts: Vec<String> = Vec::new();
                let mut tool_calls: Vec<ChatCompletionToolCall> = Vec::new();
                let mut reasoning_parts: Vec<String> = Vec::new();

                for p in &m.content {
                    match p {
                        ContentPart::Text(t) => text_parts.push(t.text.clone()),
                        ContentPart::ToolCall(tc) => {
                            tool_calls.push(ChatCompletionToolCall {
                                id: tc.id.clone(),
                                call_type: "function".to_string(),
                                function: ChatCompletionFunctionCall {
                                    name: tc.name.clone(),
                                    arguments: if tc.input.is_string() {
                                        tc.input.as_str().unwrap_or("").to_string()
                                    } else {
                                        tc.input.to_string()
                                    },
                                },
                            });
                        }
                        ContentPart::Reasoning(r) => {
                            if !r.text.is_empty() {
                                reasoning_parts.push(r.text.clone());
                            }
                        }
                        _ => {}
                    }
                }

                let content = if text_parts.is_empty() { None } else { Some(text_parts.join("\n")) };
                let tool_calls = if tool_calls.is_empty() { None } else { Some(tool_calls) };

                messages.push(ChatCompletionMessage {
                    role: ChatCompletionRole::Assistant,
                    content,
                    name: None,
                    tool_calls,
                    tool_call_id: None,
                });

                // Note: reasoning_content is not yet a field on ChatCompletionMessage.
                // For DeepSeek compatibility, we'd need to add it. For now, reasoning
                // is passed back as regular text (DeepSeek may warn but works).
                let _ = reasoning_parts; // TODO: add reasoning_content field
            }
            MessageRole::Tool => {
                // Direct tool role message (already formatted).
                let content: String = m.content.iter().filter_map(|p| match p {
                    ContentPart::Text(t) => Some(t.text.clone()),
                    _ => None,
                }).collect();
                messages.push(ChatCompletionMessage {
                    role: ChatCompletionRole::Tool,
                    content: if content.is_empty() { None } else { Some(content) },
                    name: None, tool_calls: None, tool_call_id: None,
                });
            }
        }
    }

    let gen = req.generation.as_ref();
    let tools: Option<Vec<ChatCompletionTool>> = if req.tools.is_empty() {
        None
    } else {
        Some(req.tools.iter().map(|t| ChatCompletionTool {
            tool_type: "function".to_string(),
            function: ChatCompletionToolFunction {
                name: t.name.clone(),
                description: if t.description.is_empty() { None } else { Some(t.description.clone()) },
                parameters: t.input_schema.clone(),
            },
        }).collect())
    };
    let tool_choice = if tools.is_some() {
        Some(ChatCompletionToolChoice::Mode("auto".to_string()))
    } else {
        None
    };
    ChatCompletionRequest {
        model: req.model.id.clone(),
        messages,
        temperature: gen.and_then(|g| g.temperature),
        top_p: gen.and_then(|g| g.top_p),
        n: None,
        stream: Some(false),
        stop: gen.and_then(|g| g.stop.clone()).map(|s| if s.len()==1 { StopSequence::Single(s[0].clone()) } else { StopSequence::Multiple(s) }),
        max_tokens: gen.and_then(|g| g.max_tokens),
        max_completion_tokens: None,
        presence_penalty: None,
        frequency_penalty: None,
        logit_bias: None,
        user: None,
        tools,
        tool_choice,
        seed: gen.and_then(|g| g.seed),
        response_format: None,
    }
}
#[async_trait]
impl LlmProvider for OpenAICompatibleProvider {
    fn id(&self) -> &str { "openai-compatible" }
    async fn generate(&self, request: &LlmRequest) -> Result<LlmResponse, LlmError> {
        let url = format!("{}/chat/completions", self.base_url.trim_end_matches('/'));
        let resp = self.client.post(&url).bearer_auth(&self.api_key).json(&convert_request(request)).send().await.map_err(|e| LlmError::network(e.to_string()))?;
        if !resp.status().is_success() { let s=resp.status(); let t=resp.text().await.unwrap_or_default(); return Err(LlmError::provider(format!("Error {}: {}", s, t))); }
        let c: ChatCompletionResponse = resp.json().await.map_err(|e| LlmError::parse(e.to_string()))?;
        let content = c.choices.first().and_then(|c| c.message.content.clone()).unwrap_or_default();
        let usage = c.usage.map(|u| Usage { input_tokens: Some(u.prompt_tokens), output_tokens: Some(u.completion_tokens), total_tokens: Some(u.total_tokens), ..Default::default() });
        Ok(LlmResponse { message: Message::assistant(vec![ContentPart::text(content)]), events: vec![], usage, finish_reason: FinishReason::default() })
    }
    async fn stream(&self, request: &LlmRequest) -> Result<LlmResponse, LlmError> {
        let mut body = convert_request(request); body.stream = Some(true);
        let url = format!("{}/chat/completions", self.base_url.trim_end_matches('/'));
        let resp = self.client.post(&url).bearer_auth(&self.api_key).json(&body).send().await.map_err(|e| LlmError::network(e.to_string()))?;
        if !resp.status().is_success() { let s=resp.status(); let t=resp.text().await.unwrap_or_default(); return Err(LlmError::provider(format!("Error {}: {}", s, t))); }
        let mut full = String::new(); let mut usage = None;
        let mut stream = resp.bytes_stream(); let mut buf = String::new();
        while let Some(chunk) = stream.next().await {
            buf.push_str(&String::from_utf8_lossy(&chunk.map_err(|e| LlmError::network(e.to_string()))?));
            while let Some(pos) = buf.find('\n') {
                let line = buf[..pos].trim().to_string(); buf = buf[pos+1..].to_string();
                if let Some(data) = line.strip_prefix("data: ") {
                    if data == "[DONE]" { continue; }
                    if let Ok(c) = serde_json::from_str::<ChatCompletionChunk>(data) {
                        for ch in c.choices { if let Some(content) = ch.delta.content { full.push_str(&content); } }
                        if let Some(u) = c.usage { usage = Some(Usage { input_tokens: Some(u.prompt_tokens), output_tokens: Some(u.completion_tokens), total_tokens: Some(u.total_tokens), ..Default::default() }); }
                    }
                }
            }
        }
        Ok(LlmResponse { message: Message::assistant(vec![ContentPart::text(full)]), events: vec![], usage, finish_reason: FinishReason::default() })
    }

    async fn stream_events(
        &self,
        request: &LlmRequest,
        tx: tokio::sync::mpsc::Sender<Result<crate::llm::schema::LlmEvent, LlmError>>,
    ) -> Result<LlmResponse, LlmError> {
        use crate::llm::schema::LlmEvent;
        let mut body = convert_request(request); body.stream = Some(true);
        let url = format!("{}/chat/completions", self.base_url.trim_end_matches('/'));
        tracing::info!("[DBG] stream_events: POST {} model={}", url, body.model);

        // Retry: connection failures (not HTTP errors) are retried up to 3
        // times with exponential backoff. This handles transient network
        // issues, DNS failures, etc. Slow-but-working APIs are fine — the
        // timeout is 300s so a slow response is not retried, only connection
        // failures. Mirrors claude-code-best's api_retry logic.
        const MAX_RETRIES: u32 = 3;
        let mut attempt = 0u32;
        let resp = loop {
            match self.client.post(&url).bearer_auth(&self.api_key).json(&body).send().await {
                Ok(r) => break r,
                Err(e) => {
                    let is_connect = e.is_connect() || e.is_timeout();
                    if !is_connect || attempt >= MAX_RETRIES - 1 {
                        return Err(LlmError::network(e.to_string()));
                    }
                    attempt += 1;
                    let delay = std::time::Duration::from_secs(1u64 << attempt);
                    tracing::warn!("[DBG] stream_events: connection failed (attempt {}/{}), retrying in {:?}", attempt, MAX_RETRIES, delay);
                    let _ = tx.send(Err(LlmError::network(format!(
                        "Connection failed (attempt {}/{}), retrying in {}s…",
                        attempt, MAX_RETRIES, delay.as_secs()
                    )))).await;
                    tokio::time::sleep(delay).await;
                }
            }
        };
        let status = resp.status();
        tracing::info!("[DBG] stream_events: response status: {}", status);
        if !status.is_success() {
            let t = resp.text().await.unwrap_or_default();
            tracing::warn!("[DBG] stream_events: error body: {}", t);
            return Err(LlmError::provider(format!("Error {}: {}", status, t)));
        }

        // Check content-type — if the server returned HTML (e.g. HuggingFace
        // Space is sleeping), the response is not a valid SSE stream.
        let content_type = resp.headers().get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        if content_type.contains("text/html") {
            let body = resp.text().await.unwrap_or_default();
            tracing::warn!("[DBG] stream_events: got HTML instead of SSE (API endpoint may be down)");
            return Err(LlmError::provider(
                "API endpoint returned HTML instead of a stream. The server may be sleeping or unavailable. Try again in a moment.".to_string()
            ));
        }

        let text_id = uuid::Uuid::new_v4().to_string();
        let reasoning_id = uuid::Uuid::new_v4().to_string();
        let mut full = String::new();
        let mut usage: Option<Usage> = None;
        let mut text_started = false;
        let mut reasoning_started = false;
        // Accumulated tool calls being streamed in (indexed by delta.index).
        let mut pending_tool_calls: Vec<PendingToolCall> = Vec::new();

        // Emit StepStart.
        let _ = tx.send(Ok(LlmEvent::StepStart { index: 0 })).await;

        let mut stream = resp.bytes_stream();
        let mut buf = String::new();
        let mut chunk_count = 0u32;
        while let Some(chunk) = stream.next().await {
            chunk_count += 1;
            buf.push_str(&String::from_utf8_lossy(&chunk.map_err(|e| LlmError::network(e.to_string()))?));
            while let Some(pos) = buf.find('\n') {
                let line = buf[..pos].trim().to_string();
                buf = buf[pos + 1..].to_string();
                if let Some(data) = line.strip_prefix("data: ") {
                    if data == "[DONE]" {
                        continue;
                    }
                    if let Ok(c) = serde_json::from_str::<ChatCompletionChunk>(data) {
                        for ch in c.choices {
                            // Reasoning content (DeepSeek/o1-style thinking).
                            if let Some(reasoning) = ch.delta.reasoning_content {
                                if !reasoning.is_empty() {
                                    if !reasoning_started {
                                        let _ = tx.send(Ok(LlmEvent::ReasoningStart { id: reasoning_id.clone(), provider_metadata: None })).await;
                                        reasoning_started = true;
                                    }
                                    let _ = tx.send(Ok(LlmEvent::ReasoningDelta { id: reasoning_id.clone(), text: reasoning, provider_metadata: None })).await;
                                }
                            }
                            // Regular content.
                            if let Some(content) = ch.delta.content {
                                if !content.is_empty() {
                                    // Close reasoning block if it was open.
                                    if reasoning_started {
                                        let _ = tx.send(Ok(LlmEvent::ReasoningEnd { id: reasoning_id.clone(), provider_metadata: None })).await;
                                        reasoning_started = false;
                                    }
                                    if !text_started {
                                        let _ = tx.send(Ok(LlmEvent::TextStart { id: text_id.clone(), provider_metadata: None })).await;
                                        text_started = true;
                                    }
                                    let _ = tx.send(Ok(LlmEvent::TextDelta { id: text_id.clone(), text: content.clone(), provider_metadata: None })).await;
                                    full.push_str(&content);
                                }
                            }
                            // Tool calls (streaming): accumulate name + arguments,
                            // emit ToolCall when arguments JSON is complete.
                            if let Some(tool_call_deltas) = ch.delta.tool_calls {
                                for tc in tool_call_deltas {
                                    // Ensure the slot exists.
                                    while pending_tool_calls.len() <= tc.index as usize {
                                        pending_tool_calls.push(PendingToolCall::default());
                                    }
                                    let slot = &mut pending_tool_calls[tc.index as usize];
                                    tracing::info!("[DBG] tool_call delta: index={} id={:?} func={:?}", tc.index, tc.id, tc.function.as_ref().map(|f| (&f.name, &f.arguments)));
                                    if let Some(id) = tc.id {
                                        slot.id = Some(id);
                                    }
                                    if let Some(func) = tc.function {
                                        if let Some(name) = func.name {
                                            if !name.is_empty() {
                                                slot.name = Some(name);
                                            }
                                        }
                                        if let Some(args) = func.arguments {
                                            slot.arguments.push_str(&args);
                                        }
                                    }
                                }
                            }
                        }
                        if let Some(u) = c.usage {
                            usage = Some(Usage {
                                input_tokens: Some(u.prompt_tokens),
                                output_tokens: Some(u.completion_tokens),
                                total_tokens: Some(u.total_tokens),
                                ..Default::default()
                            });
                        }
                    }
                }
            }
        }

        if reasoning_started {
            let _ = tx.send(Ok(LlmEvent::ReasoningEnd { id: reasoning_id.clone(), provider_metadata: None })).await;
        }
        if text_started {
            let _ = tx.send(Ok(LlmEvent::TextEnd { id: text_id.clone(), provider_metadata: None })).await;
        }
        // Emit accumulated tool calls. The LLM streams tool_call arguments
        // incrementally; now that the stream is done, each pending tool call
        // should have a complete JSON arguments string we can parse.
        for tc in &pending_tool_calls {
            // Skip if name is missing or empty — some providers send partial
            // tool_call deltas that don't have a complete function name.
            let name = match &tc.name {
                Some(n) if !n.is_empty() => n.clone(),
                _ => {
                    tracing::warn!("[DBG] tool_call skipped: name empty (id={:?}, args={})", tc.id, tc.arguments);
                    continue;
                }
            };
            let id = tc.id.clone().unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
            // Parse arguments; if empty, use empty object.
            let input: serde_json::Value = if tc.arguments.is_empty() {
                serde_json::Value::Object(serde_json::Map::new())
            } else {
                serde_json::from_str(&tc.arguments)
                    .unwrap_or_else(|e| {
                        tracing::warn!("[DBG] tool_call args parse failed: {} (raw: {})", e, tc.arguments);
                        serde_json::Value::Object(serde_json::Map::new())
                    })
            };
            tracing::info!("[DBG] emitting ToolCall: id={} name={} args={}", id, name, tc.arguments);
            let _ = tx.send(Ok(LlmEvent::ToolCall {
                id,
                name,
                input,
                provider_executed: None,
                provider_metadata: None,
            })).await;
        }
        tracing::info!("[DBG] stream_events: done, chunks={}, text_started={}, full_len={}, tool_calls={}", chunk_count, text_started, full.len(), pending_tool_calls.len());

        Ok(LlmResponse {
            message: Message::assistant(vec![ContentPart::text(full)]),
            events: vec![],
            usage,
            finish_reason: FinishReason::default(),
        })
    }
}
