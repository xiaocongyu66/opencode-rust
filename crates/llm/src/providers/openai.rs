//! OpenAI provider — implements Chat Completions API v1.
//!
//! Compatible with `POST https://api.openai.com/v1/chat/completions`.

use async_trait::async_trait;
use futures::StreamExt;
use crate::openai_api::*;
use crate::provider::LlmProvider;
use crate::schema::{LlmError, LlmRequest, LlmResponse, Message, MessageRole, ContentPart};

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
            ChatCompletionMessage {
                role: match m.role {
                    MessageRole::System => ChatCompletionRole::System,
                    MessageRole::Assistant => ChatCompletionRole::Assistant,
                    MessageRole::Tool => ChatCompletionRole::Tool,
                    MessageRole::User => ChatCompletionRole::User,
                },
                content: if content.is_empty() { None } else { Some(content) },
                name: None,
                tool_calls: None,
                tool_call_id: None,
            }
        }).collect();

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
            tools: None,
            tool_choice: None,
            seed,
            response_format: None,
        }
    }

    fn convert_response(&self, resp: ChatCompletionResponse) -> LlmResponse {
        let choice = resp.choices.first();
        let content = choice
            .and_then(|c| c.message.content.clone())
            .unwrap_or_default();
        let finish_reason = choice
            .and_then(|c| c.finish_reason.clone())
            .unwrap_or_default();

        let usage = resp.usage.map(|u| crate::schema::Usage {
            input_tokens: Some(u.prompt_tokens),
            output_tokens: Some(u.completion_tokens),
            total_tokens: Some(u.total_tokens),
            ..Default::default()
        });

        LlmResponse {
            message: Message::assistant(vec![ContentPart::text(content)]),
            events: vec![],
            usage,
            finish_reason: parse_finish_reason(&finish_reason),
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

fn parse_finish_reason(reason: &str) -> crate::schema::FinishReason {
    match reason {
        "stop" => crate::schema::FinishReason::Stop,
        "length" => crate::schema::FinishReason::Length,
        "tool_calls" => crate::schema::FinishReason::ToolCalls,
        "content_filter" => crate::schema::FinishReason::ContentFilter,
        _ => crate::schema::FinishReason::Unknown,
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

        let mut full_content = String::new();
        let mut finish_reason_str: Option<String> = None;
        let mut model = request.model.id.clone();
        let mut usage = None;

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
                        if !chunk.model.is_empty() {
                            model = chunk.model;
                        }
                        if let Some(u) = chunk.usage {
                            usage = Some(crate::schema::Usage {
                                input_tokens: Some(u.prompt_tokens),
                                output_tokens: Some(u.completion_tokens),
                                total_tokens: Some(u.total_tokens),
                                ..Default::default()
                            });
                        }
                        for choice in chunk.choices {
                            if let Some(content) = choice.delta.content {
                                full_content.push_str(&content);
                            }
                            if let Some(fr) = choice.finish_reason {
                                finish_reason_str = Some(fr);
                            }
                        }
                    }
                }
            }
        }

        Ok(LlmResponse {
            message: Message::assistant(vec![ContentPart::text(full_content)]),
            events: vec![],
            usage,
            finish_reason: parse_finish_reason(finish_reason_str.as_deref().unwrap_or("")),
        })
    }
}
