//! Anthropic provider — implements Messages API.
//!
//! Compatible with `POST https://api.anthropic.com/v1/messages`.

use async_trait::async_trait;
use crate::provider::LlmProvider;
use crate::schema::{ContentPart, LlmError, LlmRequest, LlmResponse, Message, MessageRole, Usage};

pub struct AnthropicProvider {
    api_key: String,
    base_url: String,
    client: reqwest::Client,
}

impl AnthropicProvider {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            base_url: "https://api.anthropic.com".to_string(),
            client: reqwest::Client::new(),
        }
    }

    pub fn from_env() -> Option<Self> {
        let api_key = std::env::var("ANTHROPIC_API_KEY").ok()?;
        Some(Self::new(api_key))
    }

    fn extract_text(&self, parts: &[ContentPart]) -> String {
        parts.iter()
            .filter_map(|p| match p {
                ContentPart::Text(t) => Some(t.text.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("")
    }
}

#[async_trait]
impl LlmProvider for AnthropicProvider {
    fn id(&self) -> &str { "anthropic" }

    async fn generate(&self, request: &LlmRequest) -> Result<LlmResponse, LlmError> {
        let system = request.messages.iter()
            .find(|m| m.role == MessageRole::System)
            .map(|m| self.extract_text(&m.content))
            .unwrap_or_default();

        let messages: Vec<serde_json::Value> = request.messages.iter()
            .filter(|m| m.role != MessageRole::System)
            .map(|m| {
                let content = self.extract_text(&m.content);
                serde_json::json!({
                    "role": match m.role {
                        MessageRole::Assistant => "assistant",
                        MessageRole::Tool => "user",
                        _ => "user",
                    },
                    "content": content,
                })
            })
            .collect();

        let max_tokens = request.generation.as_ref()
            .and_then(|g| g.max_tokens)
            .unwrap_or(4096);

        let body = serde_json::json!({
            "model": request.model.id,
            "max_tokens": max_tokens,
            "system": system,
            "messages": messages,
            "temperature": request.generation.as_ref().and_then(|g| g.temperature),
            "top_p": request.generation.as_ref().and_then(|g| g.top_p),
            "stream": false,
        });

        let url = format!("{}/v1/messages", self.base_url);
        let resp = self.client
            .post(&url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&body)
            .send()
            .await
            .map_err(|e| LlmError::network(e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(LlmError::provider(format!("Anthropic API error {}: {}", status, text)));
        }

        let result: serde_json::Value = resp.json().await
            .map_err(|e| LlmError::parse(format!("Failed to parse Anthropic response: {}", e)))?;

        let content = result["content"]
            .as_array()
            .and_then(|arr| arr.first())
            .and_then(|c| c["text"].as_str())
            .unwrap_or_default()
            .to_string();

        let usage = Usage {
            input_tokens: result["usage"]["input_tokens"].as_u64(),
            output_tokens: result["usage"]["output_tokens"].as_u64(),
            total_tokens: result["usage"]["input_tokens"].as_u64()
                .zip(result["usage"]["output_tokens"].as_u64())
                .map(|(i, o)| i + o),
            ..Default::default()
        };

        let finish_reason = result["stop_reason"].as_str()
            .map(|s| match s {
                "end_turn" => crate::schema::FinishReason::Stop,
                "max_tokens" => crate::schema::FinishReason::Length,
                "tool_use" => crate::schema::FinishReason::ToolCalls,
                _ => crate::schema::FinishReason::Unknown,
            })
            .unwrap_or_default();

        Ok(LlmResponse {
            message: Message::assistant(vec![ContentPart::text(content)]),
            events: vec![],
            usage: Some(usage),
            finish_reason,
        })
    }

    async fn stream(&self, request: &LlmRequest) -> Result<LlmResponse, LlmError> {
        self.generate(request).await
    }
}
