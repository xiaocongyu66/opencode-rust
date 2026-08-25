//! Placeholder provider implementation.

use async_trait::async_trait;
use crate::provider::LlmProvider;
use crate::schema::{LlmError, LlmRequest, LlmResponse};

pub struct Provider;

impl Provider {
    pub fn new() -> Self { Self }
    pub fn from_env() -> Option<Self> { Some(Self) }
}

#[async_trait]
impl LlmProvider for Provider {
    fn id(&self) -> &str { "placeholder" }
    async fn generate(&self, _req: &LlmRequest) -> Result<LlmResponse, LlmError> {
        Err(LlmError::provider("Not yet implemented"))
    }
    async fn stream(&self, _req: &LlmRequest) -> Result<LlmResponse, LlmError> {
        Err(LlmError::provider("Not yet implemented"))
    }
}
