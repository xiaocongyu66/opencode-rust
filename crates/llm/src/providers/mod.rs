//! LLM provider implementations.

pub mod openai;
pub mod anthropic;
pub mod openai_compatible;
pub mod google;
pub mod azure;
pub mod amazon_bedrock;
pub mod openrouter;
pub mod cloudflare;
pub mod github_copilot;
pub mod xai;

/// Registry of all built-in provider IDs.
pub const ALL_PROVIDER_IDS: &[&str] = &[
    "openai",
    "anthropic",
    "google",
    "google-vertex",
    "github-copilot",
    "amazon-bedrock",
    "azure",
    "openrouter",
    "cloudflare",
    "xai",
    "openai-compatible",
];
