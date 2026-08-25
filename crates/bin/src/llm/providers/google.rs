//! Google Gemini provider.
use async_trait::async_trait;
use crate::llm::provider::LlmProvider;
use crate::llm::schema::{ContentPart, FinishReason, LlmError, LlmRequest, LlmResponse, Message, MessageRole};

pub struct GoogleProvider { api_key: String, client: reqwest::Client }
impl GoogleProvider {
    pub fn from_env() -> Option<Self> { Some(Self { api_key: std::env::var("GOOGLE_API_KEY").ok()?, client: reqwest::Client::new() }) }
}
#[async_trait]
impl LlmProvider for GoogleProvider {
    fn id(&self) -> &str { "google" }
    async fn generate(&self, request: &LlmRequest) -> Result<LlmResponse, LlmError> {
        let model = &request.model.id;
        let url = format!("https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}", model, self.api_key);
        let contents: Vec<serde_json::Value> = request.messages.iter().filter(|m| m.role != MessageRole::System).map(|m| {
            serde_json::json!({
                "role": match m.role { MessageRole::Assistant => "model", _ => "user" },
                "parts": m.content.iter().filter_map(|p| match p { ContentPart::Text(t) => Some(serde_json::json!({"text": t.text})), _ => None }).collect::<Vec<_>>(),
            })
        }).collect();
        let system = request.messages.iter().find(|m| m.role == MessageRole::System).map(|m| {
            let text: String = m.content.iter().filter_map(|p| match p { ContentPart::Text(t) => Some(t.text.clone()), _ => None }).collect();
            serde_json::json!({ "system_instruction": { "parts": [{ "text": text }] } })
        }).unwrap_or(serde_json::json!({}));
        let mut body = serde_json::json!({
            "contents": contents,
            "generationConfig": {
                "maxOutputTokens": request.generation.as_ref().and_then(|g| g.max_tokens),
                "temperature": request.generation.as_ref().and_then(|g| g.temperature),
            }
        });
        if let Some(obj) = body.as_object_mut() {
            if let Some(sys_obj) = system.as_object() {
                for (k, v) in sys_obj {
                    obj.insert(k.clone(), v.clone());
                }
            }
        }
        let resp = self.client.post(&url).json(&body).send().await.map_err(|e| LlmError::network(e.to_string()))?;
        if !resp.status().is_success() { let s=resp.status(); let t=resp.text().await.unwrap_or_default(); return Err(LlmError::provider(format!("Google error {}: {}", s, t))); }
        let result: serde_json::Value = resp.json().await.map_err(|e| LlmError::parse(e.to_string()))?;
        let content = result["candidates"][0]["content"]["parts"][0]["text"].as_str().unwrap_or_default().to_string();
        Ok(LlmResponse { message: Message::assistant(vec![ContentPart::text(content)]), events: vec![], usage: None, finish_reason: FinishReason::default() })
    }
    async fn stream(&self, request: &LlmRequest) -> Result<LlmResponse, LlmError> { self.generate(request).await }
}
