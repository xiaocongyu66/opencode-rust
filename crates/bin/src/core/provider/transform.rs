//! Provider transform utilities.
//!
//! Ported from `provider/transform.ts`.
//! Handles schema transformation, message normalization, and provider options.

use std::collections::HashMap;

use crate::schema::ids::ProviderID;

/// Default max output tokens.
pub const OUTPUT_TOKEN_MAX: u64 = 32_000;

/// Sanitize invalid surrogate pairs in content.
pub fn sanitize_surrogates(content: &str) -> String {
    let mut result = String::with_capacity(content.len());
    let mut chars = content.chars().peekable();
    while let Some(c) = chars.next() {
        if (c as u32) >= 0xD800 && (c as u32) <= 0xDBFF {
            if let Some(&next) = chars.peek() {
                if (next as u32) >= 0xDC00 && (next as u32) <= 0xDFFF {
                    result.push(c);
                    result.push(next);
                    chars.next();
                    continue;
                }
            }
            result.push('\u{FFFD}');
        } else if (c as u32) >= 0xDC00 && (c as u32) <= 0xDFFF {
            result.push('\u{FFFD}');
        } else {
            result.push(c);
        }
    }
    result
}

/// Map npm package name to SDK key for providerOptions.
pub fn sdk_key(npm: &str) -> Option<&'static str> {
    match npm {
        "@ai-sdk/github-copilot" => Some("copilot"),
        "@ai-sdk/azure" => Some("azure"),
        "@ai-sdk/openai" => Some("openai"),
        "@ai-sdk/amazon-bedrock/mantle" => Some("openai"),
        "@ai-sdk/amazon-bedrock" => Some("bedrock"),
        "@ai-sdk/anthropic" | "@ai-sdk/google-vertex/anthropic" => Some("anthropic"),
        "@ai-sdk/google-vertex" => Some("vertex"),
        "@ai-sdk/google" => Some("google"),
        "@ai-sdk/alibaba" => Some("alibaba"),
        "@ai-sdk/cerebras" => Some("cerebras"),
        "@ai-sdk/cohere" => Some("cohere"),
        "@ai-sdk/deepinfra" => Some("deepinfra"),
        "@ai-sdk/groq" => Some("groq"),
        "@ai-sdk/mistral" => Some("mistral"),
        "@ai-sdk/perplexity" => Some("perplexity"),
        "@ai-sdk/togetherai" => Some("togetherai"),
        "@ai-sdk/vercel" => Some("vercel"),
        "@ai-sdk/xai" => Some("xai"),
        "venice-ai-sdk-provider" => Some("venice"),
        "@ai-sdk/gateway" => Some("gateway"),
        "@openrouter/ai-sdk-provider" => Some("openrouter"),
        _ => None,
    }
}

/// Check if a model is from the Kimi/Moonshot family.
pub fn is_kimi_family(provider_id: &str, model_id: &str, url: &str) -> bool {
    let lower_id = format!("{}/{}", provider_id, model_id).to_lowercase();
    if lower_id.contains("kimi") || lower_id.contains("moonshot") {
        return true;
    }
    let lower_url = url.to_lowercase();
    ["api.kimi.com", "api.moonshot.ai", "api.moonshot.cn", "api.moonshotai.cn"]
        .iter()
        .any(|host| lower_url.contains(host))
}

/// Calculate max output tokens for a model.
pub fn max_output_tokens(model_output_limit: Option<u64>, output_token_max: Option<u64>) -> u64 {
    let from_model = model_output_limit.unwrap_or(0);
    let from_flag = output_token_max.unwrap_or(OUTPUT_TOKEN_MAX);
    if from_model > 0 && from_model < from_flag {
        from_model
    } else {
        from_flag
    }
}

/// Transform a JSON schema for provider compatibility.
pub fn transform_schema(
    schema: &serde_json::Value,
    _provider_id: &ProviderID,
) -> serde_json::Value {
    let mut result = schema.clone();
    if let Some(obj) = result.as_object_mut() {
        if !obj.contains_key("additionalProperties") {
            obj.insert(
                "additionalProperties".to_string(),
                serde_json::Value::Bool(false),
            );
        }
    }
    result
}

/// Build provider options map.
pub fn provider_options(
    provider_id: &ProviderID,
    options: &HashMap<String, serde_json::Value>,
) -> HashMap<String, serde_json::Value> {
    let mut result = HashMap::new();
    if let Some(key) = sdk_key(&format!("@ai-sdk/{}", provider_id.as_str())) {
        result.insert(key.to_string(), serde_json::to_value(options).unwrap_or_default());
    }
    result
}
