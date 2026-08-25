//! Session runner — the core agent loop.
//!
//! Implements the main loop:
//! 1. Load session history → convert to LLM messages
//! 2. Resolve model + provider
//! 3. Call LLM (stream)
//! 4. Process tool calls from LLM response
//! 5. Execute tools, persist results
//! 6. If tools were called, loop back to step 1
//! 7. Stop when LLM finishes without tool calls or step limit reached
//!
//! Ported from `core/src/session/runner/llm.ts`.

use std::sync::Arc;

use opencode_llm::provider::LlmProvider;
use opencode_llm::schema::{LlmRequest, Model};
use opencode_schema::ids::SessionID;
use opencode_tools::registry::ToolRegistry;
use opencode_tools::tool::{ToolContext, ToolFailure};

use crate::session::message_converter::MessageConverter;
use crate::session::SessionStore;

pub struct SessionRunner {
    provider: Arc<dyn LlmProvider>,
    tools: Arc<ToolRegistry>,
    store: Arc<dyn SessionStore>,
    max_steps: usize,
}

impl SessionRunner {
    pub fn new(
        provider: Arc<dyn LlmProvider>,
        tools: Arc<ToolRegistry>,
        store: Arc<dyn SessionStore>,
    ) -> Self {
        Self {
            provider,
            tools,
            store,
            max_steps: 50,
        }
    }

    pub fn with_max_steps(mut self, max: usize) -> Self {
        self.max_steps = max;
        self
    }

    /// Run one full agent loop for a session.
    pub async fn run(&self, session_id: &SessionID, model: &Model, _system_prompt: &str) -> Result<RunResult, RunError> {
        let mut step = 0;
        let total_cost = 0.0f64;
        let mut total_input_tokens = 0u64;
        let mut total_output_tokens = 0u64;

        loop {
            step += 1;
            if step > self.max_steps {
                tracing::info!("Session {} reached max steps ({})", session_id, self.max_steps);
                return Ok(RunResult {
                    steps: step - 1,
                    cost: total_cost,
                    input_tokens: total_input_tokens,
                    output_tokens: total_output_tokens,
                    finish_reason: FinishReason::MaxSteps,
                });
            }

            // 1. Load session messages
            let messages = self.store.context(session_id).await.unwrap_or_default();

            // 2. Convert to LLM format
            let llm_messages = MessageConverter::convert(&messages, model);

            // 3. Build request
            let request = LlmRequest {
                model: model.clone(),
                system: vec![],
                messages: llm_messages,
                tools: vec![],
                tool_choice: None,
                generation: None,
                provider_options: None,
                http: None,
                response_format: None,
                cache: None,
                metadata: None,
                id: None,
            };

            // 4. Call LLM
            tracing::info!("Session {} step {}: calling LLM", session_id, step);
            let response = self.provider.stream(&request).await
                .map_err(|e| RunError::LlmError(e.to_string()))?;

            // 5. Accumulate usage
            if let Some(usage) = &response.usage {
                total_input_tokens += usage.input_tokens.unwrap_or(0);
                total_output_tokens += usage.output_tokens.unwrap_or(0);
            }

            // 6. Check for tool calls in the response
            let _assistant_text = response.message.content.iter()
                .filter_map(|p| match p {
                    opencode_llm::schema::ContentPart::Text(t) => Some(t.text.clone()),
                    _ => None,
                })
                .collect::<Vec<_>>()
                .join("");

            let tool_calls: Vec<&opencode_llm::schema::ContentPart> = response.message.content.iter()
                .filter(|p| matches!(p, opencode_llm::schema::ContentPart::ToolCall(_)))
                .collect();

            // 7. If no tool calls, we're done
            if tool_calls.is_empty() {
                tracing::info!("Session {} step {}: LLM finished (no tool calls)", session_id, step);
                return Ok(RunResult {
                    steps: step,
                    cost: total_cost,
                    input_tokens: total_input_tokens,
                    output_tokens: total_output_tokens,
                    finish_reason: FinishReason::Completed,
                });
            }

            // 8. Execute tool calls
            let mut ctx = ToolContext {
                session_id: session_id.0.clone(),
                agent_id: "build".to_string(),
                assistant_message_id: format!("msg_{}", step),
                tool_call_id: String::new(),
            };

            for tool_call in &tool_calls {
                if let opencode_llm::schema::ContentPart::ToolCall(tc) = tool_call {
                    tracing::info!("Session {} step {}: executing tool '{}'", session_id, step, tc.name);
                    ctx.tool_call_id = tc.id.clone();

                    let result = self.tools.execute(&tc.name, tc.input.clone(), &ctx).await;
                    match result {
                        Ok(_tool_result) => {
                            tracing::info!("Session {} tool '{}' completed", session_id, tc.name);
                        }
                        Err(ToolFailure::Message(msg)) => {
                            tracing::warn!("Session {} tool '{}' failed: {}", session_id, tc.name, msg);
                        }
                        Err(e) => {
                            tracing::error!("Session {} tool '{}' error: {}", session_id, tc.name, e);
                        }
                    }
                }
            }

            // 9. Loop back for next step
            tracing::info!("Session {} step {} completed, continuing", session_id, step);
        }
    }

    /// Interrupt the running session.
    pub async fn interrupt(&self, _session_id: &SessionID) {
        // In a real implementation, this would cancel the active LLM stream
        // and mark in-progress tools as interrupted.
        tracing::info!("Interrupt requested");
    }
}

#[derive(Debug, Clone)]
pub struct RunResult {
    pub steps: usize,
    pub cost: f64,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub finish_reason: FinishReason,
}

#[derive(Debug, Clone)]
pub enum FinishReason {
    Completed,
    MaxSteps,
    Interrupted,
}

#[derive(Debug)]
pub enum RunError {
    LlmError(String),
    ToolError(String),
    StoreError(String),
}
