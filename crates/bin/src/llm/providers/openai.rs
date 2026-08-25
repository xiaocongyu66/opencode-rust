//! OpenAI provider — implements Chat Completions API v1.
//!
//! Compatible with `POST https://api.openai.com/v1/chat/completions`.

use async_trait::async_trait;
use futures::StreamExt;
use crate::llm::openai_api::*;
use crate::llm::provider::LlmProvider;
use crate::llm::schema::{
    ContentPart, FinishReason, LlmError, LlmEvent, LlmRequest, LlmResponse, Message, MessageRole,
    ToolCallPart, ToolResultPart, ToolResultValue, Usage,
};

pub struct OpenAIProvider {
    api_key: String,
    base_url: String,
    client: reqwest::Client,
}

impl OpenAIProvider {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            base_url: "https://api.openai.com/v1".to_string(),
            client: reqwest::Client::new(),
        }
    }

    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }

    pub fn from_env() -> Option<Self> {
        let api_key = std::env::var("OPENAI_API_KEY").ok()?;
        let base_url = std::env::var("OPENAI_BASE_URL")
            .unwrap_or_else(|_| "https://api.openai.com/v1".to_string());
        Some(Self {
            api_key,
            base_url,
            client: reqwest::Client::new(),
        })
    }

    fn convert_request(&self, req: &LlmRequest) -> ChatCompletionRequest {
        let messages: Vec<ChatCompletionMessage> = req.messages.iter().map(|m| {
            let content = extract_text_content(&m.content);
            let tool_calls = extract_tool_calls(&m.content);
            let tool_call_id = extract_tool_call_id(&m.content);

            ChatCompletionMessage {
                role: match m.role {
                    MessageRole::System => ChatCompletionRole::System,
                    MessageRole::Assistant => ChatCompletionRole::Assistant,
                    MessageRole::Tool => ChatCompletionRole::Tool,
                    MessageRole::User => ChatCompletionRole::User,
                },
                content: if content.is_empty() { None } else { Some(content) },
                name: None,
                tool_calls: if tool_calls.is_empty() { None } else { Some(tool_calls) },
                tool_call_id,
            }
        }).collect();

        let tools: Option<Vec<ChatCompletionTool>> = if req.tools.is_empty() {
            None
        } else {
            Some(req.tools.iter().map(|t| ChatCompletionTool {
                tool_type: "function".to_string(),
                function: ChatCompletionToolFunction {
                    name: t.name.clone(),
                    description: Some(t.description.clone()),
                    parameters: t.input_schema.clone(),
                },
            }).collect())
        };

        let tool_choice = req.tool_choice.as_ref().map(|tc| match tc.r#type {
            crate::llm::schema::ToolChoiceType::Auto => ChatCompletionToolChoice::Mode("auto".to_string()),
            crate::llm::schema::ToolChoiceType::None => ChatCompletionToolChoice::Mode("none".to_string()),
            crate::llm::schema::ToolChoiceType::Required => ChatCompletionToolChoice::Mode("required".to_string()),
            crate::llm::schema::ToolChoiceType::Tool => ChatCompletionToolChoice::Specific(ChatCompletionToolChoiceSpecific {
                choice_type: "function".to_string(),
                function: ChatCompletionToolChoiceFunction {
                    name: tc.name.clone().unwrap_or_default(),
                },
            }),
        });

        let gen = req.generation.as_ref();
        let max_tokens = gen.and_then(|g| g.max_tokens);
        let temperature = gen.and_then(|g| g.temperature);
        let top_p = gen.and_then(|g| g.top_p);
        let stop = gen.and_then(|g| g.stop.clone()).map(|s| {
            if s.len() == 1 { StopSequence::Single(s[0].clone()) }
            else { StopSequence::Multiple(s) }
        });
        let seed = gen.and_then(|g| g.seed);
        let presence_penalty = gen.and_then(|g| g.presence_penalty);
        let frequency_penalty = gen.and_then(|g| g.frequency_penalty);

        ChatCompletionRequest {
            model: req.model.id.clone(),
            messages,
            temperature,
            top_p,
            n: None,
            stream: Some(false),
            stop,
            max_tokens,
            max_completion_tokens: None,
            presence_penalty,
            frequency_penalty,
            logit_bias: None,
            user: None,
            tools,
            tool_choice,
            seed,
            response_format: None,
        }
    }

    fn convert_response(&self, resp: ChatCompletionResponse) -> LlmResponse {
        let choice = resp.choices.first();
        let message_content = choice
            .and_then(|c| c.message.content.clone())
            .unwrap_or_default();
        let tool_calls = choice
            .and_then(|c| c.message.tool_calls.clone())
            .unwrap_or_default();
        let finish_reason = choice
            .and_then(|c| c.finish_reason.clone())
            .unwrap_or_default();

        let mut events = Vec::new();
        let mut content_parts: Vec<ContentPart> = Vec::new();

        if !message_content.is_empty() {
            let text_id = format!("text_{}", uuid::Uuid::new_v4());
            events.push(LlmEvent::TextStart {
                id: text_id.clone(),
                provider_metadata: None,
            });
            events.push(LlmEvent::TextDelta {
                id: text_id.clone(),
                text: message_content.clone(),
                provider_metadata: None,
            });
            events.push(LlmEvent::TextEnd {
                id: text_id,
                provider_metadata: None,
            });
            content_parts.push(ContentPart::text(message_content));
        }

        for tc in &tool_calls {
            let input: serde_json::Value = serde_json::from_str(&tc.function.arguments)
                .unwrap_or(serde_json::Value::Null);
            events.push(LlmEvent::ToolCall {
                id: tc.id.clone(),
                name: tc.function.name.clone(),
                input,
                provider_executed: None,
                provider_metadata: None,
            });
            content_parts.push(ContentPart::ToolCall(ToolCallPart {
                id: tc.id.clone(),
                name: tc.function.name.clone(),
                input: serde_json::from_str(&tc.function.arguments).unwrap_or(serde_json::Value::Null),
                provider_executed: None,
                metadata: None,
                provider_metadata: None,
            }));
        }

        let usage = resp.usage.map(|u| Usage {
            input_tokens: Some(u.prompt_tokens),
            output_tokens: Some(u.completion_tokens),
            total_tokens: Some(u.total_tokens),
            ..Default::default()
        });

        let fr = parse_finish_reason(&finish_reason);
        events.push(LlmEvent::StepFinish {
            index: 0,
            reason: fr.clone(),
            usage: usage.clone(),
            provider_metadata: None,
        });
        events.push(LlmEvent::Finish {
            reason: fr.clone(),
            usage: usage.clone(),
            provider_metadata: None,
        });

        LlmResponse {
            message: Message::assistant(content_parts),
            events,
            usage,
            finish_reason: fr,
        }
    }
}

fn extract_text_content(parts: &[ContentPart]) -> String {
    parts.iter()
        .filter_map(|p| match p {
            ContentPart::Text(t) => Some(t.text.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("")
}

fn extract_tool_calls(parts: &[ContentPart]) -> Vec<ChatCompletionToolCall> {
    parts.iter()
        .filter_map(|p| match p {
            ContentPart::ToolCall(tc) => Some(ChatCompletionToolCall {
                id: tc.id.clone(),
                call_type: "function".to_string(),
                function: ChatCompletionFunctionCall {
                    name: tc.name.clone(),
                    arguments: serde_json::to_string(&tc.input).unwrap_or_default(),
                },
            }),
            _ => None,
        })
        .collect()
}

fn extract_tool_call_id(parts: &[ContentPart]) -> Option<String> {
    parts.iter().find_map(|p| match p {
        ContentPart::ToolResult(tr) => Some(tr.id.clone()),
        _ => None,
    })
}

fn parse_finish_reason(reason: &str) -> FinishReason {
    match reason {
        "stop" => FinishReason::Stop,
        "length" => FinishReason::Length,
        "tool_calls" => FinishReason::ToolCalls,
        "content_filter" => FinishReason::ContentFilter,
        _ => FinishReason::Unknown,
    }
}

#[async_trait]
impl LlmProvider for OpenAIProvider {
    fn id(&self) -> &str { "openai" }

    async fn generate(&self, request: &LlmRequest) -> Result<LlmResponse, LlmError> {
        let body = self.convert_request(request);
        let url = format!("{}/chat/completions", self.base_url);

        let resp = self.client
            .post(&url)
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await
            .map_err(|e| LlmError::network(e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(LlmError::provider(format!("OpenAI API error {}: {}", status, text)));
        }

        let completion: ChatCompletionResponse = resp.json().await
            .map_err(|e| LlmError::parse(format!("Failed to parse OpenAI response: {}", e)))?;

        Ok(self.convert_response(completion))
    }

    async fn stream(&self, request: &LlmRequest) -> Result<LlmResponse, LlmError> {
        let mut body = self.convert_request(request);
        body.stream = Some(true);
        let url = format!("{}/chat/completions", self.base_url);

        let resp = self.client
            .post(&url)
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await
            .map_err(|e| LlmError::network(e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(LlmError::provider(format!("OpenAI API error {}: {}", status, text)));
        }

        let mut events = Vec::new();
        let mut content_parts: Vec<ContentPart> = Vec::new();
        let mut full_content = String::new();
        let mut finish_reason_str: Option<String> = None;
        let mut usage = None;
        let mut text_started = false;
        let text_id = format!("text_{}", uuid::Uuid::new_v4());

        let mut tool_call_accumulators: std::collections::BTreeMap<u32, ToolCallAccumulator> = std::collections::BTreeMap::new();

        let mut stream = resp.bytes_stream();
        let mut buffer = String::new();

        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result.map_err(|e| LlmError::network(e.to_string()))?;
            buffer.push_str(&String::from_utf8_lossy(&chunk));

            while let Some(pos) = buffer.find('\n') {
                let line = buffer[..pos].trim().to_string();
                buffer = buffer[pos + 1..].to_string();

                if let Some(data) = line.strip_prefix("data: ") {
                    if data == "[DONE]" { continue; }
                    if let Ok(chunk) = serde_json::from_str::<ChatCompletionChunk>(data) {
                        if let Some(u) = chunk.usage {
                            usage = Some(Usage {
                                input_tokens: Some(u.prompt_tokens),
                                output_tokens: Some(u.completion_tokens),
                                total_tokens: Some(u.total_tokens),
                                ..Default::default()
                            });
                        }
                        for choice in chunk.choices {
                            if let Some(content) = choice.delta.content {
                                if !text_started {
                                    events.push(LlmEvent::TextStart {
                                        id: text_id.clone(),
                                        provider_metadata: None,
                                    });
                                    text_started = true;
                                }
                                events.push(LlmEvent::TextDelta {
                                    id: text_id.clone(),
                                    text: content.clone(),
                                    provider_metadata: None,
                                });
                                full_content.push_str(&content);
                            }
                            if let Some(tool_calls) = choice.delta.tool_calls {
                                for tc in tool_calls {
                                    let acc = tool_call_accumulators
                                        .entry(tc.index)
                                        .or_insert_with(|| ToolCallAccumulator::new());
                                    if let Some(id) = tc.id {
                                        acc.id = id;
                                    }
                                    if let Some(func) = tc.function {
                                        if let Some(name) = func.name {
                                            acc.name = name;
                                        }
                                        if let Some(args) = func.arguments {
                                            acc.arguments.push_str(&args);
                                        }
                                    }
                                }
                            }
                            if let Some(fr) = choice.finish_reason {
                                finish_reason_str = Some(fr);
                            }
                        }
                    }
                }
            }
        }

        if text_started {
            events.push(LlmEvent::TextEnd {
                id: text_id,
                provider_metadata: None,
            });
            content_parts.push(ContentPart::text(full_content));
        }

        for (_, acc) in tool_call_accumulators.iter() {
            if acc.id.is_empty() {
                continue;
            }
            let input: serde_json::Value = serde_json::from_str(&acc.arguments)
                .unwrap_or(serde_json::Value::Null);
            events.push(LlmEvent::ToolCall {
                id: acc.id.clone(),
                name: acc.name.clone(),
                input: input.clone(),
                provider_executed: None,
                provider_metadata: None,
            });
            content_parts.push(ContentPart::ToolCall(ToolCallPart {
                id: acc.id.clone(),
                name: acc.name.clone(),
                input,
                provider_executed: None,
                metadata: None,
                provider_metadata: None,
            }));
        }

        let fr = parse_finish_reason(finish_reason_str.as_deref().unwrap_or(""));
        events.push(LlmEvent::StepFinish {
            index: 0,
            reason: fr.clone(),
            usage: usage.clone(),
            provider_metadata: None,
        });
        events.push(LlmEvent::Finish {
            reason: fr.clone(),
            usage: usage.clone(),
            provider_metadata: None,
        });

        Ok(LlmResponse {
            message: Message::assistant(content_parts),
            events,
            usage,
            finish_reason: fr,
        })
    }

    async fn stream_events(
        &self,
        request: &LlmRequest,
        tx: tokio::sync::mpsc::Sender<Result<LlmEvent, LlmError>>,
    ) -> Result<LlmResponse, LlmError> {
        let response = self.stream(request).await?;
        for event in &response.events {
            let _ = tx.send(Ok(event.clone())).await;
        }
        Ok(response)
    }
}

struct ToolCallAccumulator {
    id: String,
    name: String,
    arguments: String,
}

impl ToolCallAccumulator {
    fn new() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            arguments: String::new(),
        }
    }
}
