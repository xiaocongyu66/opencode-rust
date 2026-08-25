//! AWS Bedrock provider.
use async_trait::async_trait;
use crate::llm::provider::LlmProvider;
use crate::llm::schema::{ContentPart, FinishReason, LlmError, LlmRequest, LlmResponse, Message, MessageRole};

pub struct BedrockProvider { access_key: String, secret_key: String, region: String, client: reqwest::Client }
impl BedrockProvider {
    pub fn from_env() -> Option<Self> {
        Some(Self {
            access_key: std::env::var("AWS_ACCESS_KEY_ID").ok()?,
            secret_key: std::env::var("AWS_SECRET_ACCESS_KEY").ok()?,
            region: std::env::var("AWS_REGION").unwrap_or_else(|_| "us-east-1".to_string()),
            client: reqwest::Client::new(),
        })
    }
}
#[async_trait]
impl LlmProvider for BedrockProvider {
    fn id(&self) -> &str { "amazon-bedrock" }
    async fn generate(&self, request: &LlmRequest) -> Result<LlmResponse, LlmError> {
        let model = &request.model.id;
        let url = format!("https://bedrock-runtime.{}.amazonaws.com/model/{}/invoke", self.region, model);
        let messages: Vec<serde_json::Value> = request.messages.iter().map(|m| {
            let content: String = m.content.iter().filter_map(|p| match p { ContentPart::Text(t) => Some(t.text.clone()), _ => None }).collect();
            serde_json::json!({ "role": match m.role { MessageRole::Assistant => "assistant", MessageRole::System => "system", _ => "user" }, "content": [{ "type": "text", "text": content }] })
        }).collect();
        let body = serde_json::json!({ "anthropic_version": "bedrock-2023-05-31", "messages": messages, "max_tokens": request.generation.as_ref().and_then(|g| g.max_tokens).unwrap_or(4096) });
        let resp = self.client.post(&url).header("x-amz-access-key", &self.access_key).header("x-amz-secret-key", &self.secret_key).json(&body).send().await.map_err(|e| LlmError::network(e.to_string()))?;
        if !resp.status().is_success() { let s=resp.status(); let t=resp.text().await.unwrap_or_default(); return Err(LlmError::provider(format!("Bedrock error {}: {}", s, t))); }
        let result: serde_json::Value = resp.json().await.map_err(|e| LlmError::parse(e.to_string()))?;
        let content = result["content"][0]["text"].as_str().unwrap_or_default().to_string();
        Ok(LlmResponse { message: Message::assistant(vec![ContentPart::text(content)]), events: vec![], usage: None, finish_reason: FinishReason::default() })
    }
    async fn stream(&self, request: &LlmRequest) -> Result<LlmResponse, LlmError> { self.generate(request).await }
}
