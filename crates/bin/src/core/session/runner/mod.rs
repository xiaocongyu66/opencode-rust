//! Session runner module — the core agent loop.
//!
//! Ported from `core/src/session/runner/`:
//! - `index.ts` → runner orchestration
//! - `llm.ts` → provider turn + tool settlement loop
//! - `model.ts` → model resolution
//! - `to-llm-message.ts` → message converter
//! - `publish-llm-event.ts` → event publisher
//! - `max-steps.ts` → max-steps prompt constant

pub mod max_steps;
pub mod message_converter;
pub mod model;
pub mod publish_llm_event;
pub mod runner;

pub use runner::{RunError, RunResult, FinishReason, SessionRunner, RunnerEvent};
pub use model::{ModelResolver, ModelResolveError};
pub use publish_llm_event::{LlmEventPublisher, PublisherInput};
