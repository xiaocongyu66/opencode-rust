//! Session runner — the core agent loop.
//!
//! Ported from `core/src/session/runner/llm.ts`.
//! Implements the main loop:
//! 1. Load session history, convert to LLM messages
//! 2. Resolve model + provider
//! 3. Call LLM (stream events)
//! 4. Process tool calls from LLM response
//! 5. Execute tools, persist results
//! 6. If tools were called, loop back to step 1
//! 7. Stop when LLM finishes without tool calls or step limit reached

use std::sync::Arc;

use tokio::sync::mpsc;

use crate::llm::provider::LlmProvider;
use crate::llm::schema::{
    LlmEvent, LlmRequest, Message, Model, SystemPart, ToolChoice,
    ToolChoiceType, Usage,
};
use crate::schema::ids::SessionID;
use crate::schema::session::SessionInfo;
use crate::tools::registry::ToolRegistry;
use crate::tools::tool::ToolContext;

use super::max_steps::MAX_STEPS_PROMPT;
use super::message_converter::to_llm_messages;
use super::model::ModelResolver;
use super::publish_llm_event::{LlmEventPublisher, PublisherInput};

#[derive(Debug, thiserror::Error)]
pub enum RunError {
    #[error("LLM error: {0}")]
    LlmError(String),
    #[error("Model resolve error: {0}")]
    ModelError(String),
    #[error("Tool error: {0}")]
    ToolError(String),
    #[error("Store error: {0}")]
    StoreError(String),
    #[error("Session not found: {0}")]
    SessionNotFound(String),
    #[error("Interrupted")]
    Interrupted,
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

struct TurnResult {
    needs_continuation: bool,
    usage: Option<Usage>,
}

/// Events emitted by the runner for real-time TUI display.
#[derive(Debug, Clone)]
pub enum RunnerEvent {
    /// A new provider turn is starting.
    StepStarted { step: usize },
    /// Incremental text delta from the LLM.
    TextDelta { text: String },
    /// The LLM's text output for this turn is complete.
    TextDone { text: String },
    /// Incremental reasoning (thinking) delta from the LLM.
    ReasoningDelta { text: String },
    /// The LLM's reasoning output is complete.
    ReasoningDone { text: String },
    /// The LLM is starting a tool call.
    ToolStarted {
        tool_name: String,
        call_id: String,
        input: serde_json::Value,
    },
    /// A tool call completed successfully.
    ToolSuccess {
        tool_name: String,
        call_id: String,
        summary: String,
    },
    /// A tool call failed.
    ToolFailed {
        tool_name: String,
        call_id: String,
        error: String,
    },
    /// A provider turn finished.
    StepFinished {
        step: usize,
        finish_reason: String,
        /// Cumulative tokens used so far in this run.
        usage: Option<Usage>,
    },
    /// Token usage crossed a compaction threshold (claude-code-book Ch07).
    /// The TUI shows a warning; AutoCompact triggers at the next tier.
    CompactionNeeded {
        tier: crate::core::session::compaction::CompactionTier,
        used: u64,
        effective: u64,
    },
    /// The runner encountered an error.
    Error { message: String },
    /// The entire run is complete.
    Done { result: RunResult },
}

pub struct SessionRunner {
    provider: Arc<dyn LlmProvider>,
    tools: Arc<ToolRegistry>,
    store: Arc<dyn crate::core::session::SessionStore>,
    model_resolver: Arc<dyn ModelResolver>,
    max_steps: usize,
}

impl SessionRunner {
    pub fn new(
        provider: Arc<dyn LlmProvider>,
        tools: Arc<ToolRegistry>,
        store: Arc<dyn crate::core::session::SessionStore>,
        model_resolver: Arc<dyn ModelResolver>,
    ) -> Self {
        Self {
            provider,
            tools,
            store,
            model_resolver,
            max_steps: 50,
        }
    }

    pub fn with_max_steps(mut self, max: usize) -> Self {
        self.max_steps = max;
        self
    }

    /// Run the agent loop with streaming events sent to `tx`.
    pub async fn run_with_events(
        &self,
        session_id: &SessionID,
        system_prompt: &str,
        agent_id: &str,
        agent_steps: Option<u64>,
        tx: mpsc::Sender<RunnerEvent>,
    ) -> Result<RunResult, RunError> {
        self.run_inner(session_id, system_prompt, agent_id, agent_steps, Some(tx)).await
    }

    /// Run the agent loop without streaming events.
    pub async fn run(
        &self,
        session_id: &SessionID,
        system_prompt: &str,
        agent_id: &str,
        agent_steps: Option<u64>,
    ) -> Result<RunResult, RunError> {
        self.run_inner(session_id, system_prompt, agent_id, agent_steps, None).await
    }

    async fn run_inner(
        &self,
        session_id: &SessionID,
        system_prompt: &str,
        agent_id: &str,
        agent_steps: Option<u64>,
        tx: Option<mpsc::Sender<RunnerEvent>>,
    ) -> Result<RunResult, RunError> {
        tracing::info!("[DBG] run_inner: start, session={}", session_id.0);
        // Ensure the session exists — create a stub if not found.
        let session = match self.store.get(session_id).await {
            Some(s) => {
                tracing::info!("[DBG] run_inner: session found");
                s
            }
            None => {
                tracing::info!("[DBG] run_inner: session not found, creating stub");
                use crate::schema::session::{SessionInfo, SessionTokens, SessionTime};
                use crate::schema::ids::{ProjectID, AgentID};
                use crate::schema::location::LocationRef;
                use crate::schema::common::AbsolutePath;
                let cwd = std::env::current_dir()
                    .map(|p| AbsolutePath(p.to_string_lossy().to_string()))
                    .unwrap_or_else(|_| AbsolutePath(String::from("/")));
                let info = SessionInfo {
                    id: session_id.clone(),
                    parent_id: None,
                    project_id: ProjectID::from_str("default"),
                    agent: Some(AgentID(agent_id.to_string())),
                    model: None,
                    cost: 0.0,
                    tokens: SessionTokens::default(),
                    time: SessionTime {
                        created: chrono::Utc::now(),
                        updated: chrono::Utc::now(),
                        archived: None,
                    },
                    title: crate::core::session::default_parent_title(),
                    location: LocationRef {
                        directory: cwd,
                        workspace_id: None,
                    },
                    subpath: None,
                    revert: None,
                };
                self.store.create(info).await
            }
        };
        let _ = session;

        let model = self
            .model_resolver
            .resolve(&session)
            .map_err(|e| RunError::ModelError(e.to_string()))?;

        // Effective context window (claude-code-book Ch07): model window
        // minus the reserve for the compaction LLM call itself.
        let model_window = model
            .defaults
            .as_ref()
            .and_then(|d| d.limits.as_ref())
            .and_then(|l| l.context)
            .unwrap_or(200_000);
        let max_output = model
            .defaults
            .as_ref()
            .and_then(|d| d.limits.as_ref())
            .and_then(|l| l.output)
            .unwrap_or(8_192);
        let effective = crate::core::session::compaction::effective_window(model_window, max_output);
        let mut breaker = crate::core::session::compaction::CircuitBreaker::default();

        let effective_max_steps = agent_steps.map(|s| s as usize).unwrap_or(self.max_steps);

        let mut step = 0usize;
        let total_cost = 0.0f64;
        let mut total_input_tokens = 0u64;
        let mut total_output_tokens = 0u64;

        loop {
            step += 1;
            let is_last_step = step >= effective_max_steps;
            if is_last_step {
                tracing::info!("Session {} reached max steps ({})", session_id, effective_max_steps);
            }

            // Check compaction tier before each step (claude-code-book Ch07).
            // Skip when breaker is Open or when we have no tx to report on.
            let used = total_input_tokens + total_output_tokens;
            let tier = crate::core::session::compaction::pick_tier(used, effective);
            if tier != crate::core::session::compaction::CompactionTier::None {
                if breaker.should_try() {
                    if let Some(tx) = &tx {
                        let _ = tx.send(RunnerEvent::CompactionNeeded {
                            tier,
                            used,
                            effective,
                        }).await;
                    }
                }
                if tier == crate::core::session::compaction::CompactionTier::Blocking {
                    tracing::warn!(
                        session_id = %session_id,
                        used, effective,
                        "context budget blocking; stopping run"
                    );
                    // In Blocking tier we refuse to start a new step to avoid
                    // overflowing the model window. The actual LLM summarization
                    // is a follow-up; for now emit Done and return.
                    let run_result = RunResult {
                        steps: step.saturating_sub(1),
                        cost: total_cost,
                        input_tokens: total_input_tokens,
                        output_tokens: total_output_tokens,
                        finish_reason: FinishReason::Interrupted,
                    };
                    if let Some(tx) = &tx {
                        let _ = tx.send(RunnerEvent::Done { result: run_result.clone() }).await;
                    }
                    return Ok(run_result);
                }
            }

            if let Some(tx) = &tx {
                let _ = tx.send(RunnerEvent::StepStarted { step }).await;
            }

            let result = self
                .run_turn(session_id, &session, &model, system_prompt, agent_id, step, is_last_step, tx.as_ref())
                .await?;

            if let Some(usage) = &result.usage {
                total_input_tokens += usage.input_tokens.unwrap_or(0);
                total_output_tokens += usage.output_tokens.unwrap_or(0);
            }

            if let Some(tx) = &tx {
                let _ = tx.send(RunnerEvent::StepFinished {
                    step,
                    finish_reason: if is_last_step { "max_steps".to_string() } else { "completed".to_string() },
                    usage: result.usage.clone(),
                }).await;
            }

            if !result.needs_continuation {
                let finish_reason = if is_last_step {
                    FinishReason::MaxSteps
                } else {
                    FinishReason::Completed
                };
                let run_result = RunResult {
                    steps: step,
                    cost: total_cost,
                    input_tokens: total_input_tokens,
                    output_tokens: total_output_tokens,
                    finish_reason: finish_reason.clone(),
                };
                if let Some(tx) = &tx {
                    let _ = tx.send(RunnerEvent::Done { result: run_result.clone() }).await;
                }
                return Ok(run_result);
            }

            if is_last_step {
                let run_result = RunResult {
                    steps: step,
                    cost: total_cost,
                    input_tokens: total_input_tokens,
                    output_tokens: total_output_tokens,
                    finish_reason: FinishReason::MaxSteps,
                };
                if let Some(tx) = &tx {
                    let _ = tx.send(RunnerEvent::Done { result: run_result.clone() }).await;
                }
                return Ok(run_result);
            }

            tracing::info!("Session {} step {} completed, continuing", session_id, step);
        }
    }

    async fn run_turn(
        &self,
        session_id: &SessionID,
        _session: &SessionInfo,
        model: &Model,
        system_prompt: &str,
        agent_id: &str,
        step: usize,
        is_last_step: bool,
        tx: Option<&mpsc::Sender<RunnerEvent>>,
    ) -> Result<TurnResult, RunError> {
        let messages = self.store.context(session_id).await.unwrap_or_default();
        let mut llm_messages = to_llm_messages(&messages, model);

        if is_last_step {
            llm_messages.push(Message::assistant(vec![crate::llm::schema::ContentPart::text(
                MAX_STEPS_PROMPT,
            )]));
        }

        let tool_defs: Vec<crate::llm::schema::ToolDefinition> = if is_last_step {
            vec![]
        } else {
            self.tools
                .definitions()
                .into_iter()
                .map(|d| crate::llm::schema::ToolDefinition {
                    name: d.name,
                    description: d.description,
                    input_schema: d.input_schema,
                    output_schema: None,
                    cache: None,
                    metadata: None,
                    native: None,
                })
                .collect()
        };

        let system_parts = if system_prompt.is_empty() {
            vec![]
        } else {
            // Mark the system prompt as cacheable. Anthropic-style providers
            // reuse this prefix across turns (95-99% cache hit per claude-code-
            // book Ch13). Non-supporting providers ignore the hint.
            let mut part = SystemPart::new(system_prompt);
            part.cache = Some(crate::llm::schema::CacheHint {
                r#type: crate::llm::schema::CacheHintType::Ephemeral,
                ttl_seconds: None,
            });
            vec![part]
        };

        let request = LlmRequest {
            id: None,
            model: model.clone(),
            system: system_parts,
            messages: llm_messages,
            tools: tool_defs,
            tool_choice: if is_last_step {
                Some(ToolChoice {
                    r#type: ToolChoiceType::None,
                    name: None,
                })
            } else {
                None
            },
            generation: None,
            provider_options: None,
            http: None,
            response_format: None,
            cache: None,
            metadata: None,
        };

        tracing::info!("Session {} step {}: calling LLM", session_id, step);

        let publisher_input = PublisherInput {
            session_id: session_id.clone(),
            agent: agent_id.to_string(),
            model: crate::schema::model::ModelRef {
                id: crate::schema::ids::ModelID(model.id.clone()),
                provider_id: crate::schema::ids::ProviderID(model.provider.clone()),
                variant: None,
            },
            snapshot: None,
        };
        let publisher = Arc::new(tokio::sync::Mutex::new(LlmEventPublisher::new(publisher_input)));

        let (event_tx, mut event_rx) = mpsc::channel::<Result<LlmEvent, crate::llm::schema::LlmError>>(256);

        let provider = self.provider.clone();
        let request_clone = request.clone();
        let stream_handle = tokio::spawn(async move {
            provider.stream_events(&request_clone, event_tx).await
        });

        let tools = self.tools.clone();
        let store_clone = self.store.clone();
        let session_id_clone = session_id.clone();
        let agent_id_clone = agent_id.to_string();
        let tx_clone = tx.cloned();

        // Load hook registry from ~/.rsopencode/hooks.json (claude-code-book Ch08).
        // PreToolUse hooks can deny a tool call before it runs; PostToolUse hooks
        // fire after completion. Missing file = no hooks, proceed normally.
        let hooks = load_hooks();
        let step_clone = step;
        let publisher_for_task = publisher.clone();
        let model_clone = model.clone();

        let tool_task = tokio::spawn(async move {
            let mut accumulated_text = String::new();
            let mut accumulated_reasoning = String::new();
            let mut needs_continuation = false;
            let mut last_usage: Option<Usage> = None;

            while let Some(event_result) = event_rx.recv().await {
                let event = match event_result {
                    Ok(e) => e,
                    Err(e) => {
                        if let Some(tx) = &tx_clone {
                            let _ = tx.send(RunnerEvent::Error { message: e.to_string() }).await;
                        }
                        continue;
                    }
                };

                match &event {
                    LlmEvent::TextDelta { text, .. } => {
                        accumulated_text.push_str(text);
                        if let Some(tx) = &tx_clone {
                            let _ = tx.send(RunnerEvent::TextDelta { text: text.clone() }).await;
                        }
                    }
                    LlmEvent::TextEnd { .. } => {
                        if !accumulated_text.is_empty() {
                            if let Some(tx) = &tx_clone {
                                let _ = tx.send(RunnerEvent::TextDone { text: accumulated_text.clone() }).await;
                            }
                            accumulated_text.clear();
                        }
                    }
                    LlmEvent::ReasoningDelta { text, .. } => {
                        accumulated_reasoning.push_str(text);
                        if let Some(tx) = &tx_clone {
                            let _ = tx.send(RunnerEvent::ReasoningDelta { text: text.clone() }).await;
                        }
                    }
                    LlmEvent::ReasoningEnd { .. } => {
                        if !accumulated_reasoning.is_empty() {
                            if let Some(tx) = &tx_clone {
                                let _ = tx.send(RunnerEvent::ReasoningDone { text: accumulated_reasoning.clone() }).await;
                            }
                            accumulated_reasoning.clear();
                        }
                    }
                    LlmEvent::ToolCall { id, name, input, provider_executed, .. } => {
                        if provider_executed != &Some(true) {
                            needs_continuation = true;
                            // Store the assistant message with tool call so the
                            // next turn sees the complete tool_call → tool_result
                            // pair (required by OpenAI API format).
                            {
                                use crate::schema::session::{SessionMessage, AssistantContent, AssistantTime, AssistantToolTime, AssistantToolProvider};
                                use crate::schema::ids::SessionMessageID;
                                let input_str = serde_json::to_string(&input).unwrap_or_default();
                                let msg = SessionMessage::Assistant {
                                    id: SessionMessageID::new(),
                                    metadata: None,
                                    time: AssistantTime {
                                        created: chrono::Utc::now(),
                                        completed: None,
                                    },
                                    agent: agent_id_clone.clone(),
                                    model: crate::schema::model::ModelRef {
                                        id: crate::schema::ids::ModelID(model_clone.id.clone()),
                                        provider_id: crate::schema::ids::ProviderID(model_clone.provider.clone()),
                                        variant: None,
                                    },
                                    content: vec![AssistantContent::Tool {
                                        id: id.clone(),
                                        name: name.clone(),
                                        provider: Some(AssistantToolProvider {
                                            executed: false,
                                            metadata: None,
                                            result_metadata: None,
                                        }),
                                        state: crate::schema::session::ToolState::Running {
                                            input: serde_json::from_str(&input_str).unwrap_or_default(),
                                            structured: std::collections::HashMap::new(),
                                            content: vec![],
                                        },
                                        time: AssistantToolTime {
                                            created: chrono::Utc::now(),
                                            ran: Some(chrono::Utc::now()),
                                            completed: None,
                                            pruned: None,
                                        },
                                    }],
                                    snapshot: None,
                                    finish: None,
                                    cost: None,
                                    tokens: None,
                                    error: None,
                                };
                                let _ = store_clone.append_message(&session_id_clone, msg).await;
                            }
                            if let Some(tx) = &tx_clone {
                                let _ = tx.send(RunnerEvent::ToolStarted {
                                    tool_name: name.clone(),
                                    call_id: id.clone(),
                                    input: input.clone(),
                                }).await;
                            }

                            let ctx = ToolContext {
                                session_id: session_id_clone.0.clone(),
                                agent_id: agent_id_clone.clone(),
                                assistant_message_id: id.clone(),
                                tool_call_id: id.clone(),
                            };

                            tracing::info!(
                                "Session {} step {}: executing tool '{}'",
                                session_id_clone,
                                step_clone,
                                name
                            );

                            // PreToolUse hook (claude-code-book Ch08): resolve
                            // the chain for this tool+event and run it. A deny
                            // decision short-circuits the tool call.
                            let hook_input = crate::core::hooks::HookInput {
                                event: crate::core::hooks::EVENT_PRE_TOOL_USE.to_string(),
                                tool: Some(name.clone()),
                                input: Some(input.clone()),
                                session_id: Some(session_id_clone.0.clone()),
                                cwd: std::env::current_dir().ok().map(|p| p.to_string_lossy().to_string()),
                            };
                            let hook_chain = hooks.resolve(&hook_input);
                            let pre_decision = crate::core::hooks::run_chain(&hook_chain, &hook_input).await;
                            let result = match pre_decision.decision() {
                                crate::core::hooks::HookDecision::Deny => {
                                    // Hook denied: synthesize a failure result.
                                    let reason = pre_decision.reason.clone().unwrap_or_else(|| "denied by PreToolUse hook".to_string());
                                    Err(crate::tools::tool::ToolFailure::Message(reason))
                                }
                                _ => {
                                    // Allow or passthrough: execute the tool.
                                    tools.execute(name, input.clone(), &ctx).await
                                }
                            };

                            // PostToolUse hook fires after the tool returns.
                            let post_input = crate::core::hooks::HookInput {
                                event: crate::core::hooks::EVENT_POST_TOOL_USE.to_string(),
                                tool: Some(name.clone()),
                                input: Some(input.clone()),
                                session_id: Some(session_id_clone.0.clone()),
                                cwd: std::env::current_dir().ok().map(|p| p.to_string_lossy().to_string()),
                            };
                            let post_chain = hooks.resolve(&post_input);
                            let _ = crate::core::hooks::run_chain(&post_chain, &post_input).await;

                            let result = result;
                            match &result {
                                Ok(tool_result) => {
                                    let summary = tool_result.content.iter()
                                        .filter_map(|c| match c {
                                            crate::tools::tool::ToolContent::Text { text } => Some(text.as_str()),
                                            _ => None,
                                        })
                                        .collect::<Vec<_>>()
                                        .join("\n");
                                    // Pass up to 8000 chars of output to the LLM so it has
                                    // enough context to continue. Truncate by chars (not bytes)
                                    // to avoid panicking on multi-byte UTF-8 boundaries.
                                    let truncated = if summary.chars().count() > 8000 {
                                        let head: String = summary.chars().take(8000).collect();
                                        format!("{}\n... (output truncated, {} total chars)", head, summary.chars().count())
                                    } else {
                                        summary
                                    };
                                    if let Some(tx) = &tx_clone {
                                        let _ = tx.send(RunnerEvent::ToolSuccess {
                                            tool_name: name.clone(),
                                            call_id: id.clone(),
                                            summary: truncated.clone(),
                                        }).await;
                                    }
                                    // Store the tool result so the LLM sees it
                                    // in the next turn's context. Without this
                                    // the LLM doesn't know the tool ran and
                                    // re-calls it repeatedly.
                                    {
                                        use crate::schema::session::SessionMessage;
                                        use crate::schema::ids::SessionMessageID;
                                        let msg = SessionMessage::Shell {
                                            id: SessionMessageID::new(),
                                            metadata: None,
                                            time: crate::schema::session::ShellTime {
                                                created: chrono::Utc::now(),
                                                completed: Some(chrono::Utc::now()),
                                            },
                                            call_id: id.clone(),
                                            command: name.clone(),
                                            output: truncated,
                                        };
                                        let _ = store_clone.append_message(&session_id_clone, msg).await;
                                    }
                                    tracing::info!(
                                        "Session {} tool '{}' completed",
                                        session_id_clone,
                                        name
                                    );
                                }
                                Err(e) => {
                                    if let Some(tx) = &tx_clone {
                                        let _ = tx.send(RunnerEvent::ToolFailed {
                                            tool_name: name.clone(),
                                            call_id: id.clone(),
                                            error: e.to_string(),
                                        }).await;
                                    }
                                    tracing::warn!(
                                        "Session {} tool '{}' failed: {}",
                                        session_id_clone,
                                        name,
                                        e
                                    );
                                }
                            }
                        }
                    }
                    LlmEvent::StepFinish { usage, .. } => {
                        if let Some(u) = usage {
                            last_usage = Some(u.clone());
                        }
                    }
                    LlmEvent::Finish { usage, .. } => {
                        if let Some(u) = usage {
                            last_usage = Some(u.clone());
                        }
                    }
                    LlmEvent::ProviderError { message, .. } => {
                        if let Some(tx) = &tx_clone {
                            let _ = tx.send(RunnerEvent::Error { message: message.clone() }).await;
                        }
                    }
                    _ => {}
                }

                let pub_clone = publisher_for_task.clone();
                let _ = pub_clone.lock().await.publish(&event);
            }

            let pub_clone = publisher_for_task.clone();
            let _ = pub_clone.lock().await.flush();
            (needs_continuation, last_usage)
        });

        let stream_result = stream_handle.await
            .map_err(|e| RunError::LlmError(format!("Stream task panicked: {}", e)))?
            .map_err(|e| RunError::LlmError(e.to_string()))?;

        let (needs_continuation, usage) = tool_task.await
            .map_err(|e| RunError::LlmError(format!("Tool task panicked: {}", e)))?;

        let publisher_clone = publisher.clone();
        let has_error = publisher_clone.lock().await.has_provider_error();
        Ok(TurnResult {
            needs_continuation: needs_continuation && !has_error,
            usage: usage.or(stream_result.usage.clone()),
        })
    }

    pub async fn interrupt(&self, _session_id: &SessionID) {
        tracing::info!("Interrupt requested");
    }
}

/// Load hook registry from ~/.rsopencode/hooks.json at the user layer.
/// Returns an empty registry if the file is missing or unreadable.
fn load_hooks() -> crate::core::hooks::HookRegistry {
    use crate::core::hooks::{HookRegistry, HookLayer};
    let mut reg = HookRegistry::new();
    let path = match dirs::home_dir() {
        Some(h) => h.join(".rsopencode").join("hooks.json"),
        None => return reg,
    };
    if let Err(e) = reg.load_file(&path, HookLayer::User) {
        tracing::debug!("hooks load skipped: {}", e);
    }
    reg
}
