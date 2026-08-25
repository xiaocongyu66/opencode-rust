//! OpenAI-compatible provider — uses Chat Completions API format.

use async_trait::async_trait;
use crate::llm::openai_api::*;
use crate::llm::provider::LlmProvider;
use crate::llm::schema::{ContentPart, FinishReason, LlmError, LlmRequest, LlmResponse, Message, MessageRole, Usage};
use futures::StreamExt;

pub struct OpenRouterProvider { api_key: String, client: reqwest::Client }
impl OpenRouterProvider {
    pub fn from_env() -> Option<Self> { Some(Self { api_key: std::env::var("OPENROUTER_API_KEY").ok()?, client: reqwest::Client::new() }) }
}
fn convert_request(req: &LlmRequest) -> ChatCompletionRequest {
    let messages: Vec<ChatCompletionMessage> = req.messages.iter().map(|m| {
        let content: String = m.content.iter().filter_map(|p| match p { ContentPart::Text(t) => Some(t.text.clone()), _ => None }).collect();
        ChatCompletionMessage { role: match m.role { MessageRole::System => ChatCompletionRole::System, MessageRole::Assistant => ChatCompletionRole::Assistant, MessageRole::Tool => ChatCompletionRole::Tool, _ => ChatCompletionRole::User }, content: if content.is_empty() { None } else { Some(content) }, name: None, tool_calls: None, tool_call_id: None }
    }).collect();
    let gen = req.generation.as_ref();
    ChatCompletionRequest { model: req.model.id.clone(), messages, temperature: gen.and_then(|g| g.temperature), top_p: gen.and_then(|g| g.top_p), n: None, stream: Some(false), stop: gen.and_then(|g| g.stop.clone()).map(|s| if s.len()==1 { StopSequence::Single(s[0].clone()) } else { StopSequence::Multiple(s) }), max_tokens: gen.and_then(|g| g.max_tokens), max_completion_tokens: None, presence_penalty: None, frequency_penalty: None, logit_bias: None, user: None, tools: None, tool_choice: None, seed: gen.and_then(|g| g.seed), response_format: None }
}
#[async_trait]
impl LlmProvider for OpenRouterProvider {
    fn id(&self) -> &str { "openrouter" }
    async fn generate(&self, request: &LlmRequest) -> Result<LlmResponse, LlmError> {
        let url = "https://openrouter.ai/api/v1/chat/completions";
        let resp = self.client.post(url).bearer_auth(&self.api_key).json(&convert_request(request)).send().await.map_err(|e| LlmError::network(e.to_string()))?;
        if !resp.status().is_success() { let s = resp.status(); let t = resp.text().await.unwrap_or_default(); return Err(LlmError::provider(format!("OpenRouter error {}: {}", s, t))); }
        let completion: ChatCompletionResponse = resp.json().await.map_err(|e| LlmError::parse(e.to_string()))?;
        let content = completion.choices.first().and_then(|c| c.message.content.clone()).unwrap_or_default();
        let usage = completion.usage.map(|u| Usage { input_tokens: Some(u.prompt_tokens), output_tokens: Some(u.completion_tokens), total_tokens: Some(u.total_tokens), ..Default::default() });
        Ok(LlmResponse { message: Message::assistant(vec![ContentPart::text(content)]), events: vec![], usage, finish_reason: FinishReason::default() })
    }
    async fn stream(&self, request: &LlmRequest) -> Result<LlmResponse, LlmError> {
        let mut body = convert_request(request); body.stream = Some(true);
        let url = "https://openrouter.ai/api/v1/chat/completions";
        let resp = self.client.post(url).bearer_auth(&self.api_key).json(&body).send().await.map_err(|e| LlmError::network(e.to_string()))?;
        if !resp.status().is_success() { let s = resp.status(); let t = resp.text().await.unwrap_or_default(); return Err(LlmError::provider(format!("OpenRouter error {}: {}", s, t))); }
        let mut full = String::new();
        let mut usage = None;
        let mut stream = resp.bytes_stream();
        let mut buf = String::new();
        while let Some(chunk) = stream.next().await {
            buf.push_str(&String::from_utf8_lossy(&chunk.map_err(|e| LlmError::network(e.to_string()))?));
            while let Some(pos) = buf.find('\n') {
                let line = buf[..pos].trim().to_string();
                buf = buf[pos + 1..].to_string();
                if let Some(data) = line.strip_prefix("data: ") {
                    if data == "[DONE]" {
                        continue;
                    }
                    if let Ok(chunk) = serde_json::from_str::<ChatCompletionChunk>(data) {
                        for c in chunk.choices {
                            if let Some(content) = c.delta.content {
                                full.push_str(&content);
                            }
                        }
                        if let Some(u) = chunk.usage {
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
        Ok(LlmResponse { message: Message::assistant(vec![ContentPart::text(full)]), events: vec![], usage, finish_reason: FinishReason::default() })
    }
}
