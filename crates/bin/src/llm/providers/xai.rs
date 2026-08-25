//! xAI (Grok) provider — OpenAI-compatible.
use async_trait::async_trait;
use crate::llm::openai_api::*;
use crate::llm::provider::LlmProvider;
use crate::llm::schema::{ContentPart, FinishReason, LlmError, LlmRequest, LlmResponse, Message, MessageRole, Usage};
use futures::StreamExt;

pub struct XaiProvider { api_key: String, client: reqwest::Client }
impl XaiProvider {
    pub fn from_env() -> Option<Self> { Some(Self { api_key: std::env::var("XAI_API_KEY").ok()?, client: reqwest::Client::new() }) }
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
impl LlmProvider for XaiProvider {
    fn id(&self) -> &str { "xai" }
    async fn generate(&self, request: &LlmRequest) -> Result<LlmResponse, LlmError> {
        let resp = self.client.post("https://api.x.ai/v1/chat/completions").bearer_auth(&self.api_key).json(&convert_request(request)).send().await.map_err(|e| LlmError::network(e.to_string()))?;
        if !resp.status().is_success() { let s=resp.status(); let t=resp.text().await.unwrap_or_default(); return Err(LlmError::provider(format!("xAI error {}: {}", s, t))); }
        let c: ChatCompletionResponse = resp.json().await.map_err(|e| LlmError::parse(e.to_string()))?;
        let content = c.choices.first().and_then(|c| c.message.content.clone()).unwrap_or_default();
        let usage = c.usage.map(|u| Usage { input_tokens: Some(u.prompt_tokens), output_tokens: Some(u.completion_tokens), total_tokens: Some(u.total_tokens), ..Default::default() });
        Ok(LlmResponse { message: Message::assistant(vec![ContentPart::text(content)]), events: vec![], usage, finish_reason: FinishReason::default() })
    }
    async fn stream(&self, request: &LlmRequest) -> Result<LlmResponse, LlmError> { self.generate(request).await }
}
