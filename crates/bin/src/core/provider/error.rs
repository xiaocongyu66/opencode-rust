//! Provider error handling.
//!
//! Ported from `provider/error.ts`.
//! Implements error parsing for API call errors and stream errors.

use std::collections::HashMap;

/// Error when provider response headers time out.
#[derive(Debug, Clone, thiserror::Error)]
#[error("Provider response headers timed out after {ms}ms")]
pub struct HeaderTimeoutError {
    pub ms: u64,
}

/// Error when the response stream fails.
#[derive(Debug, Clone, thiserror::Error)]
#[error("{0}")]
pub struct ResponseStreamError(pub String);

/// Parsed stream error.
#[derive(Debug, Clone)]
pub enum ParsedStreamError {
    ContextOverflow { message: String, response_body: String },
    ApiError {
        message: String,
        is_retryable: bool,
        response_body: String,
    },
}

/// Parsed API call error.
#[derive(Debug, Clone)]
pub enum ParsedApiCallError {
    ContextOverflow { message: String, response_body: Option<String> },
    ApiError {
        message: String,
        statusCode: Option<u32>,
        is_retryable: bool,
        response_headers: Option<HashMap<String, String>>,
        response_body: Option<String>,
        metadata: Option<HashMap<String, String>>,
    },
}

/// Check if a message indicates context overflow.
fn is_context_overflow(message: &str) -> bool {
    let lower = message.to_lowercase();
    lower.contains("context_length_exceeded")
        || lower.contains("context length")
        || lower.contains("context window")
        || lower.contains("too long")
        || lower.contains("maximum context")
}

/// Format an error message from an API call error.
pub fn format_message(
    _provider_id: &str,
    message: &str,
    status_code: Option<u32>,
    response_body: Option<&str>,
) -> String {
    let msg = if message.is_empty() {
        if let Some(body) = response_body {
            body.to_string()
        } else if let Some(code) = status_code {
            status_text(code).to_string()
        } else {
            "Unknown error".to_string()
        }
    } else if response_body.is_none() || status_code.is_some_and(|c| message != status_text(c)) {
        message.to_string()
    } else {
        if let Some(body) = response_body {
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(body) {
                let err_msg = parsed
                    .pointer("/message")
                    .or_else(|| parsed.pointer("/error/message"))
                    .and_then(|v| v.as_str());
                if let Some(err) = err_msg {
                    return format!("{}: {}", message, err);
                }
            }
        }
        message.to_string()
    };
    msg.trim().to_string()
}

fn status_text(code: u32) -> &'static str {
    match code {
        400 => "Bad Request",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "Not Found",
        408 => "Request Timeout",
        409 => "Conflict",
        413 => "Payload Too Large",
        429 => "Too Many Requests",
        500 => "Internal Server Error",
        502 => "Bad Gateway",
        503 => "Service Unavailable",
        504 => "Gateway Timeout",
        _ => "Unknown",
    }
}

/// Parse a stream error from raw input.
pub fn parse_stream_error(input: &serde_json::Value) -> Option<ParsedStreamError> {
    let body = input;
    if body.get("type").and_then(|v| v.as_str()) != Some("error") {
        return None;
    }

    let response_body = serde_json::to_string(body).unwrap_or_default();
    let code = body.pointer("/error/code").and_then(|v| v.as_str())?;

    match code {
        "context_length_exceeded" => Some(ParsedStreamError::ContextOverflow {
            message: "Input exceeds context window of this model".to_string(),
            response_body,
        }),
        "insufficient_quota" => Some(ParsedStreamError::ApiError {
            message: "Quota exceeded. Check your plan and billing details.".to_string(),
            is_retryable: false,
            response_body,
        }),
        "usage_not_included" => Some(ParsedStreamError::ApiError {
            message: "To use Codex with your ChatGPT plan, upgrade to Plus: https://chatgpt.com/explore/plus.".to_string(),
            is_retryable: false,
            response_body,
        }),
        "invalid_prompt" => Some(ParsedStreamError::ApiError {
            message: body
                .pointer("/error/message")
                .and_then(|v| v.as_str())
                .unwrap_or("Invalid prompt.")
                .to_string(),
            is_retryable: false,
            response_body,
        }),
        "server_is_overloaded" | "server_error" => Some(ParsedStreamError::ApiError {
            message: body
                .pointer("/error/message")
                .and_then(|v| v.as_str())
                .unwrap_or("Server error.")
                .to_string(),
            is_retryable: true,
            response_body,
        }),
        _ => None,
    }
}

/// Parse an API call error.
pub fn parse_api_call_error(
    provider_id: &str,
    message: &str,
    status_code: Option<u32>,
    is_retryable: bool,
    response_headers: Option<&HashMap<String, String>>,
    response_body: Option<&str>,
    url: Option<&str>,
) -> ParsedApiCallError {
    let m = format_message(provider_id, message, status_code, response_body);

    let body_json = response_body.and_then(|b| serde_json::from_str::<serde_json::Value>(b).ok());
    let body_code = body_json
        .as_ref()
        .and_then(|v| v.pointer("/error/code"))
        .and_then(|v| v.as_str());

    if is_context_overflow(&m) || status_code == Some(413) || body_code == Some("context_length_exceeded") {
        return ParsedApiCallError::ContextOverflow {
            message: m,
            response_body: response_body.map(|s| s.to_string()),
        };
    }

    let retryable = if provider_id.starts_with("openai") {
        status_code == Some(404) || is_retryable
    } else {
        is_retryable
    };

    let metadata = url.map(|u| {
        let mut m = HashMap::new();
        m.insert("url".to_string(), u.to_string());
        m
    });

    ParsedApiCallError::ApiError {
        message: m,
        statusCode: status_code,
        is_retryable: retryable,
        response_headers: response_headers.cloned(),
        response_body: response_body.map(|s| s.to_string()),
        metadata,
    }
}
