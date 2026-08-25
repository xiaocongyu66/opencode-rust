//! GitHub Copilot provider.
use async_trait::async_trait;
use crate::llm::openai_api::*;
use crate::llm::provider::LlmProvider;
use crate::llm::schema::{ContentPart, FinishReason, LlmError, LlmRequest, LlmResponse, Message, MessageRole};

pub struct GithubCopilotProvider { token: String, client: reqwest::Client }
impl GithubCopilotProvider {
    pub fn from_env() -> Option<Self> { Some(Self { token: std::env::var("GITHUB_COPILOT_TOKEN").ok()?, client: reqwest::Client::new() }) }
}
fn convert_request(req: &LlmRequest) -> ChatCompletionRequest {
    let messages: Vec<ChatCompletionMessage> = req.messages.iter().map(|m| {
        let content: String = m.content.iter().filter_map(|p| match p { ContentPart::Text(t) => Some(t.text.clone()), _ => None }).collect();
        ChatCompletionMessage { role: match m.role { MessageRole::System => ChatCompletionRole::System, MessageRole::Assistant => ChatCompletionRole::Assistant, _ => ChatCompletionRole::User }, content: if content.is_empty() { None } else { Some(content) }, name: None, tool_calls: None, tool_call_id: None }
    }).collect();
    ChatCompletionRequest { model: req.model.id.clone(), messages, temperature: None, top_p: None, n: None, stream: Some(false), stop: None, max_tokens: None, max_completion_tokens: None, presence_penalty: None, frequency_penalty: None, logit_bias: None, user: None, tools: None, tool_choice: None, seed: None, response_format: None }
}
#[async_trait]
impl LlmProvider for GithubCopilotProvider {
    fn id(&self) -> &str { "github-copilot" }
    async fn generate(&self, request: &LlmRequest) -> Result<LlmResponse, LlmError> {
        let resp = self.client.post("https://api.githubcopilot.com/chat/completions").bearer_auth(&self.token).json(&convert_request(request)).send().await.map_err(|e| LlmError::network(e.to_string()))?;
        if !resp.status().is_success() { let s=resp.status(); let t=resp.text().await.unwrap_or_default(); return Err(LlmError::provider(format!("Copilot error {}: {}", s, t))); }
        let c: ChatCompletionResponse = resp.json().await.map_err(|e| LlmError::parse(e.to_string()))?;
        let content = c.choices.first().and_then(|c| c.message.content.clone()).unwrap_or_default();
        Ok(LlmResponse { message: Message::assistant(vec![ContentPart::text(content)]), events: vec![], usage: None, finish_reason: FinishReason::default() })
    }
    async fn stream(&self, request: &LlmRequest) -> Result<LlmResponse, LlmError> { self.generate(request).await }
}
