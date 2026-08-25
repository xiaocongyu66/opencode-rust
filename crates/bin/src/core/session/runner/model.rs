//! Session runner model resolution.
//!
//! Ported from `core/src/session/runner/model.ts`.
//! Resolves a `SessionSchema.Info` into a canonical `Model` for the provider turn.

use crate::llm::schema::{Model, ModelDefaults, ModelLimits};
use crate::schema::ids::{ModelID, ProviderID, SessionID};
use crate::schema::model::{ModelApi, ModelInfo, ModelVariant};
use crate::schema::session::SessionInfo;

#[derive(Debug, thiserror::Error)]
pub enum ModelResolveError {
    #[error("No model is available for session {0}")]
    ModelNotSelected(SessionID),
    #[error("Model unavailable: {0}/{1}")]
    ModelUnavailable(ProviderID, ModelID),
    #[error("Variant unavailable for {0}/{1}: {2}")]
    VariantUnavailable(ProviderID, ModelID, String),
    #[error("Unsupported API for {0}/{1}: {2}")]
    UnsupportedApi(ProviderID, ModelID, String),
}

pub trait ModelResolver: Send + Sync {
    fn resolve(&self, session: &SessionInfo) -> Result<Model, ModelResolveError>;
}

fn with_variant(model: &ModelInfo, variant_id: Option<&str>) -> Result<ModelInfo, ModelResolveError> {
    let id = match variant_id {
        Some(v) if v == "default" => model.request.variant.clone(),
        Some(v) => Some(v.to_string()),
        None => model.request.variant.clone(),
    };

    let variant: Option<&ModelVariant> = model.variants.iter().find(|v| id.as_deref() == Some(v.id.0.as_str()));

    if variant.is_none() && variant_id.is_some() && variant_id != Some("default") {
        return Err(ModelResolveError::VariantUnavailable(
            model.provider_id.clone(),
            model.id.clone(),
            variant_id.unwrap().to_string(),
        ));
    }

    if let Some(v) = variant {
        let mut model = model.clone();
        for (k, val) in &v.request.headers {
            model.request.fields.headers.insert(k.clone(), val.clone());
        }
        for (k, val) in &v.request.body {
            model.request.fields.body.insert(k.clone(), val.clone());
        }
        Ok(model)
    } else {
        Ok(model.clone())
    }
}

fn api_name(model: &ModelInfo) -> String {
    match &model.api {
        ModelApi::Aisdk { package, .. } => format!("aisdk:{}", package),
        ModelApi::Native { .. } => "native".to_string(),
    }
}

pub fn from_catalog_model(model: &ModelInfo, api_key: Option<&str>) -> Result<Model, ModelResolveError> {
    let supported = match &model.api {
        ModelApi::Aisdk { package, .. } => {
            package == "@ai-sdk/openai"
                || package == "@ai-sdk/anthropic"
                || package == "@ai-sdk/openai-compatible"
        }
        ModelApi::Native { .. } => true,
    };

    if !supported {
        return Err(ModelResolveError::UnsupportedApi(
            model.provider_id.clone(),
            model.id.clone(),
            api_name(model),
        ));
    }

    let defaults = ModelDefaults {
        limits: Some(ModelLimits {
            context: Some(model.limit.context as u64),
            output: Some(model.limit.output as u64),
        }),
        generation: None,
        provider_options: None,
        http: None,
    };

    let _ = api_key;

    Ok(Model {
        id: match &model.api {
            ModelApi::Aisdk { id, .. } => id.0.clone(),
            ModelApi::Native { id, .. } => id.0.clone(),
        },
        provider: model.provider_id.0.clone(),
        defaults: Some(defaults),
        compatibility: None,
    })
}

pub fn resolve(
    session: &SessionInfo,
    model: &ModelInfo,
    api_key: Option<&str>,
) -> Result<Model, ModelResolveError> {
    let variant_id = session
        .model
        .as_ref()
        .and_then(|r| r.variant.as_ref().map(|v| v.0.as_str()))
        .map(|s| s.to_string());
    let model = with_variant(model, variant_id.as_deref())?;
    from_catalog_model(&model, api_key)
}

pub struct CatalogModelResolver<F>
where
    F: Fn(&str, &str) -> Option<ModelInfo> + Send + Sync,
{
    lookup: F,
    api_key: Option<String>,
}

impl<F> CatalogModelResolver<F>
where
    F: Fn(&str, &str) -> Option<ModelInfo> + Send + Sync,
{
    pub fn new(lookup: F, api_key: Option<String>) -> Self {
        Self { lookup, api_key }
    }
}

impl<F> ModelResolver for CatalogModelResolver<F>
where
    F: Fn(&str, &str) -> Option<ModelInfo> + Send + Sync,
{
    fn resolve(&self, session: &SessionInfo) -> Result<Model, ModelResolveError> {
        if let Some(model_ref) = &session.model {
            let model = (self.lookup)(&model_ref.provider_id.0, &model_ref.id.0).ok_or(
                ModelResolveError::ModelUnavailable(model_ref.provider_id.clone(), model_ref.id.clone()),
            )?;
            resolve(session, &model, self.api_key.as_deref())
        } else {
            Err(ModelResolveError::ModelNotSelected(session.id.clone()))
        }
    }
}

/// A simple model resolver that uses a fixed provider + model ID.
/// Useful when the provider is selected from environment variables.
pub struct EnvModelResolver {
    model_id: String,
    provider_id: String,
}

impl EnvModelResolver {
    pub fn new(model_id: impl Into<String>, provider_id: impl Into<String>) -> Self {
        Self {
            model_id: model_id.into(),
            provider_id: provider_id.into(),
        }
    }
}

impl ModelResolver for EnvModelResolver {
    fn resolve(&self, _session: &SessionInfo) -> Result<Model, ModelResolveError> {
        Ok(Model {
            id: self.model_id.clone(),
            provider: self.provider_id.clone(),
            defaults: Some(ModelDefaults {
                limits: Some(ModelLimits {
                    context: Some(128_000),
                    output: Some(16_384),
                }),
                generation: None,
                provider_options: None,
                http: None,
            }),
            compatibility: None,
        })
    }
}
