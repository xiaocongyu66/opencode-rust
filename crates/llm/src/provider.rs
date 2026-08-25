//! Provider abstraction — async trait for LLM providers.

use async_trait::async_trait;

use crate::schema::{LlmError, LlmRequest, LlmResponse, Model, ModelDefaults, ModelCompatibility};

/// Selected-model request defaults without applying precedence.
pub type ProviderModelOptions = (Option<ModelDefaults>, Option<ModelCompatibility>);

/// A provider definition: id + model factory.
pub struct ProviderDefinition {
    pub id: String,
    pub model: Box<dyn Fn(&str, ProviderModelOptions) -> Model + Send + Sync>,
}

impl std::fmt::Debug for ProviderDefinition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProviderDefinition")
            .field("id", &self.id)
            .finish_non_exhaustive()
    }
}

/// Async trait every concrete provider implements.
#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// Stable provider identifier (e.g. `"openai"`, `"anthropic"`).
    fn id(&self) -> &str;

    /// Collect a full response — same events as `stream`, gathered into an
    /// [`LlmResponse`].
    async fn generate(&self, request: &LlmRequest) -> Result<LlmResponse, LlmError>;

    /// Stream provider-neutral [`LlmEvent`]s, returning them collected into an
    /// [`LlmResponse`]. Use [`LlmProvider::stream_events`] when incremental
    /// events are wanted.
    async fn stream(&self, request: &LlmRequest) -> Result<LlmResponse, LlmError>;

    /// Stream events into a channel. Default implementation delegates to
    /// `stream` and returns the collected response; concrete providers should
    /// override this to emit events incrementally.
    async fn stream_events(
        &self,
        request: &LlmRequest,
        tx: tokio::sync::mpsc::Sender<Result<crate::schema::LlmEvent, LlmError>>,
    ) -> Result<LlmResponse, LlmError> {
        let response = self.stream(request).await?;
        for event in &response.events {
            let _ = tx.send(Ok(event.clone())).await;
        }
        Ok(response)
    }
}
