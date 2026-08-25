//! Cloudflare Workers AI provider.
use async_trait::async_trait;
use crate::llm::openai_api::*;
use crate::llm::provider::LlmProvider;
use crate::llm::schema::{ContentPart, FinishReason, LlmError, LlmRequest, LlmResponse, Message, MessageRole, Usage};

pub struct CloudflareProvider { api_key: String, account_id: String, client: reqwest::Client }
impl CloudflareProvider {
    pub fn from_env() -> Option<Self> {
        Some(Self { api_key: std::env::var("CLOUDFLARE_API_KEY").ok()?, account_id: std::env::var("CLOUDFLARE_ACCOUNT_ID").ok()?, client: reqwest::Client::new() })
    }
}
fn convert_request(req: &LlmRequest) -> ChatCompletionRequest {
    let messages: Vec<ChatCompletionMessage> = req.messages.iter().map(|m| {
        let content: String = m.content.iter().filter_map(|p| match p { ContentPart::Text(t) => Some(t.text.clone()), _ => None }).collect();
        ChatCompletionMessage { role: match m.role { MessageRole::System => ChatCompletionRole::System, MessageRole::Assistant => ChatCompletionRole::Assistant, _ => ChatCompletionRole::User }, content: if content.is_empty() { None } else { Some(content) }, name: None, tool_calls: None, tool_call_id: None }
    }).collect();
    ChatCompletionRequest { model: req.model.id.clone(), messages, temperature: None, top_p: None, n: None, stream: Some(false), stop: None, max_tokens: None, max_completion_tokens: None, presence_penalty: None, frequency_penalty: None, logit_bias: None, user: None, tools: None, tool_choice: None, seed: None, response_format: None }
}
#[async_trait]
impl LlmProvider for CloudflareProvider {
    fn id(&self) -> &str { "cloudflare" }
    async fn generate(&self, request: &LlmRequest) -> Result<LlmResponse, LlmError> {
        let url = format!("https://api.cloudflare.com/client/v4/accounts/{}/ai/v1/chat/completions", self.account_id);
        let resp = self.client.post(&url).bearer_auth(&self.api_key).json(&convert_request(request)).send().await.map_err(|e| LlmError::network(e.to_string()))?;
        if !resp.status().is_success() { let s=resp.status(); let t=resp.text().await.unwrap_or_default(); return Err(LlmError::provider(format!("Cloudflare error {}: {}", s, t))); }
        let c: ChatCompletionResponse = resp.json().await.map_err(|e| LlmError::parse(e.to_string()))?;
        let content = c.choices.first().and_then(|c| c.message.content.clone()).unwrap_or_default();
        Ok(LlmResponse { message: Message::assistant(vec![ContentPart::text(content)]), events: vec![], usage: None, finish_reason: FinishReason::default() })
    }
    async fn stream(&self, request: &LlmRequest) -> Result<LlmResponse, LlmError> { self.generate(request).await }
}
