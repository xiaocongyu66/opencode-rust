//! Provider factory — selects the right LLM provider.
//!
//! Selection order:
//! 1. TOML config (`~/.rsopencode/config.toml` or `./.rsopencode/config.toml`)
//!    → providers defined there take priority
//! 2. `OPENAI_BASE_URL` + key → OpenAI-compatible provider
//! 3. `OPENAI_API_KEY` → OpenAI provider
//! 4. `ANTHROPIC_API_KEY` → Anthropic provider
//! 5. `GOOGLE_API_KEY` → Google provider
//! 6. Default → None
//!
//! Model selection: each branch reads a `*_MODEL` env var (e.g. `OPENAI_MODEL`,
//! `ANTHROPIC_MODEL`, `GOOGLE_MODEL`) with a sensible default.

use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::llm::provider::LlmProvider;
use crate::llm::providers::anthropic::AnthropicProvider;
use crate::llm::providers::openai::OpenAIProvider;
use crate::llm::providers::google::GoogleProvider;
use crate::llm::providers::openai_compatible::OpenAICompatibleProvider;

/// Provider selection result.
pub struct ProviderSelection {
    pub provider: Arc<dyn LlmProvider>,
    pub model_id: String,
    pub provider_id: String,
    /// Human-readable provider name for display (e.g. "Murasame NewAPI").
    pub provider_name: String,
    /// Human-readable model name for display (e.g. "GLM 5.2 FP8").
    pub model_name: String,
}

/// TOML config file schema (subset relevant to provider selection).
///
/// Example:
/// ```toml
/// default_provider = "murasame"
/// default_model = "glm-5.2-fp8"
///
/// [[providers]]
/// id = "murasame"
/// name = "Murasame NewAPI"
/// base_url = "https://murasame52-newapi.hf.space/v1"
/// api_key = "sk-..."
/// kind = "openai-compatible"
///
/// [[providers.models]]
/// id = "glm-5.2-fp8"
/// name = "GLM 5.2 FP8"
///
/// [[providers.models]]
/// id = "deepseek-v4-flash"
/// name = "DeepSeek V4 Flash"
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProviderConfig {
    /// Default provider id to use.
    #[serde(default)]
    pub default_provider: Option<String>,
    /// Default model id to use.
    #[serde(default)]
    pub default_model: Option<String>,
    /// Default agent id.
    #[serde(default)]
    pub default_agent: Option<String>,
    /// Theme name.
    #[serde(default)]
    pub theme: Option<String>,
    /// Locale ("en" or "zh").
    #[serde(default)]
    pub locale: Option<String>,
    /// Provider list.
    #[serde(default)]
    pub providers: Vec<ProviderEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderEntry {
    /// Unique provider id (e.g. "murasame").
    pub id: String,
    /// Display name (e.g. "Murasame NewAPI").
    #[serde(default)]
    pub name: Option<String>,
    /// Base URL for OpenAI-compatible providers.
    #[serde(default)]
    pub base_url: Option<String>,
    /// API key.
    #[serde(default)]
    pub api_key: Option<String>,
    /// Provider kind: "openai-compatible", "openai", "anthropic", "google".
    /// Defaults to "openai-compatible".
    #[serde(default)]
    pub kind: Option<String>,
    /// Models offered by this provider.
    #[serde(default)]
    pub models: Vec<ModelEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelEntry {
    /// Model id sent to the API (e.g. "glm-5.2-fp8").
    pub id: String,
    /// Display name (e.g. "GLM 5.2 FP8"). Falls back to `id` if absent.
    #[serde(default)]
    pub name: Option<String>,
}

/// Load the TOML config from well-known locations.
/// Search order (first wins): ./.rsopencode/config.toml → ~/.rsopencode/config.toml
pub fn load_config() -> Option<ProviderConfig> {
    let candidates = config_paths();
    for path in candidates {
        if let Ok(content) = std::fs::read_to_string(&path) {
            match toml::from_str::<ProviderConfig>(&content) {
                Ok(cfg) => {
                    tracing::info!("loaded config from {}", path.display());
                    return Some(cfg);
                }
                Err(e) => {
                    tracing::warn!("failed to parse {}: {}", path.display(), e);
                }
            }
        }
    }
    None
}

/// Candidate config file paths, in priority order.
pub fn config_paths() -> Vec<std::path::PathBuf> {
    let mut paths = Vec::new();
    // Project-local config (highest priority).
    if let Ok(cwd) = std::env::current_dir() {
        paths.push(cwd.join(".rsopencode").join("config.toml"));
    }
    // Global user config.
    if let Some(home) = dirs::home_dir() {
        paths.push(home.join(".rsopencode").join("config.toml"));
    }
    paths
}

/// List all models from the config file. Returns (provider_id, model_id, display_name).
/// Empty if no config or no providers.
pub fn list_configured_models() -> Vec<(String, String, String)> {
    let mut out = Vec::new();
    if let Some(cfg) = load_config() {
        for p in &cfg.providers {
            for m in &p.models {
                let display = m.name.clone().unwrap_or_else(|| m.id.clone());
                out.push((p.id.clone(), m.id.clone(), display));
            }
        }
    }
    out
}

/// Switch the active model by id. Updates the config's default_model and
/// returns the display name (or None if not found).
pub fn switch_model(model_id: &str) -> Option<String> {
    let cfg = load_config()?;
    // Find the model across all providers.
    for p in &cfg.providers {
        for m in &p.models {
            if m.id == model_id {
                return Some(m.name.clone().unwrap_or_else(|| m.id.clone()));
            }
        }
    }
    None
}

/// Select a provider from the TOML config file. Returns None if no config
/// exists or no provider has an api_key set.
pub fn select_from_config() -> Option<ProviderSelection> {
    tracing::info!("[DBG] select_from_config: start");
    let cfg = load_config()?;
    tracing::info!("[DBG] config loaded: providers={}", cfg.providers.len());
    let default_provider_id = cfg.default_provider.as_deref()?;
    tracing::info!("[DBG] default_provider_id={:?}", default_provider_id);
    let provider = cfg
        .providers
        .iter()
        .find(|p| p.id == default_provider_id)?;
    let api_key = provider.api_key.as_deref()?.to_string();
    if api_key.is_empty() {
        return None;
    }
    let base_url = provider.base_url.clone().unwrap_or_default();
    let kind = provider.kind.as_deref().unwrap_or("openai-compatible");
    let provider_name = provider
        .name
        .clone()
        .unwrap_or_else(|| provider.id.clone());

    // Resolve the model: explicit default_model, else first model in the list.
    let model = cfg
        .default_model
        .as_deref()
        .and_then(|id| provider.models.iter().find(|m| m.id == id))
        .or_else(|| provider.models.first())?;
    let model_id = model.id.clone();
    let model_name = model
        .name
        .clone()
        .unwrap_or_else(|| model_id.clone());

    let provider_arc: Arc<dyn LlmProvider> = match kind {
        "openai" => Arc::new(OpenAIProvider::new(api_key)),
        "anthropic" => Arc::new(AnthropicProvider::new(api_key)),
        "google" => GoogleProvider::from_env()
            .map(|p| Arc::new(p) as Arc<dyn LlmProvider>)
            .unwrap_or_else(|| Arc::new(OpenAICompatibleProvider::new(base_url, api_key))),
        _ => Arc::new(OpenAICompatibleProvider::new(base_url, api_key)),
    };

    Some(ProviderSelection {
        provider: provider_arc,
        model_id,
        provider_id: provider.id.clone(),
        provider_name,
        model_name,
    })
}

/// Select an LLM provider and default model.
///
/// Tries the TOML config first, then falls back to environment variables.
/// Returns `None` if neither yields a configured provider.
pub fn select_from_env() -> Option<ProviderSelection> {
    // 1. TOML config (highest priority — user's explicit setup)
    if let Some(sel) = select_from_config() {
        return Some(sel);
    }

    // 2. OPENAI_BASE_URL → OpenAI-compatible provider (priority for custom endpoints)
    if let Ok(base_url) = std::env::var("OPENAI_BASE_URL") {
        if !base_url.is_empty() {
            let api_key = std::env::var("OPENAI_API_KEY")
                .or_else(|_| std::env::var("OPENAI_COMPATIBLE_API_KEY"))
                .or_else(|_| std::env::var("LLM_API_KEY"))
                .unwrap_or_default();
            if !api_key.is_empty() {
                let provider = OpenAICompatibleProvider::new(base_url.clone(), api_key);
                let model_id = std::env::var("OPENAI_MODEL")
                    .unwrap_or_else(|_| "gpt-4o".to_string());
                return Some(ProviderSelection {
                    provider: Arc::new(provider),
                    model_id: model_id.clone(),
                    provider_id: "openai-compatible".to_string(),
                    provider_name: "OpenAI-Compatible".to_string(),
                    model_name: model_id,
                });
            }
        }
    }

    // 3. OPENAI_API_KEY → OpenAI provider
    if let Ok(api_key) = std::env::var("OPENAI_API_KEY") {
        if !api_key.is_empty() {
            let provider = OpenAIProvider::new(api_key);
            let model_id = std::env::var("OPENAI_MODEL")
                .unwrap_or_else(|_| "gpt-4o".to_string());
            return Some(ProviderSelection {
                provider: Arc::new(provider),
                model_id: model_id.clone(),
                provider_id: "openai".to_string(),
                provider_name: "OpenAI".to_string(),
                model_name: model_id,
            });
        }
    }

    // 4. ANTHROPIC_API_KEY → Anthropic provider
    if let Ok(api_key) = std::env::var("ANTHROPIC_API_KEY") {
        if !api_key.is_empty() {
            let provider = AnthropicProvider::new(api_key);
            let model_id = std::env::var("ANTHROPIC_MODEL")
                .unwrap_or_else(|_| "claude-sonnet-4-20250514".to_string());
            return Some(ProviderSelection {
                provider: Arc::new(provider),
                model_id: model_id.clone(),
                provider_id: "anthropic".to_string(),
                provider_name: "Anthropic".to_string(),
                model_name: model_id,
            });
        }
    }

    // 5. GOOGLE_API_KEY → Google provider
    if let Ok(api_key) = std::env::var("GOOGLE_API_KEY") {
        if !api_key.is_empty() {
            let provider = GoogleProvider::from_env().unwrap();
            let model_id = std::env::var("GOOGLE_MODEL")
                .unwrap_or_else(|_| "gemini-2.0-flash".to_string());
            return Some(ProviderSelection {
                provider: Arc::new(provider),
                model_id: model_id.clone(),
                provider_id: "google".to_string(),
                provider_name: "Google".to_string(),
                model_name: model_id,
            });
        }
    }

    None
}
