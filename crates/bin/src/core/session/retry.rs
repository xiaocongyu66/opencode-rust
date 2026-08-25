//! Session retry policy.
//!
//! Ported from `session/retry.ts`.
//! Implements exponential backoff with jitter and retry-after header parsing.

use std::time::Duration;

pub const RETRY_INITIAL_DELAY: u64 = 2_000;
pub const RETRY_BACKOFF_FACTOR: u64 = 2;
pub const RETRY_JITTER_FACTOR: f64 = 0.25;
pub const RETRY_MAX_DELAY_NO_HEADERS: u64 = 30_000;
pub const RETRY_MAX_DELAY: u64 = 2_147_483_647;
pub const RETRY_MAX_RETRIES: u32 = 5;

pub const GO_UPSELL_MESSAGE: &str = "Free usage exceeded, subscribe to Go";
pub const GO_UPSELL_URL: &str = "https://opencode.ai/go";

/// Retryable error info.
#[derive(Debug, Clone)]
pub struct Retryable {
    pub message: String,
    pub action: Option<RetryAction>,
}

#[derive(Debug, Clone)]
pub struct RetryAction {
    pub reason: String,
    pub provider: String,
    pub title: String,
    pub message: String,
    pub label: String,
    pub link: Option<String>,
}

/// API error data extracted from provider errors.
#[derive(Debug, Clone, Default)]
pub struct ApiErrorData {
    pub message: String,
    pub status_code: Option<u32>,
    pub is_retryable: bool,
    pub response_headers: Option<std::collections::HashMap<String, String>>,
    pub response_body: Option<String>,
}

/// Patterns that indicate a retryable error message.
const RETRYABLE_PATTERNS: &[&str] = &[
    r"429|500|502|503|504|524",
    r"(?i)rate increased too quickly|rate limit|rate-limit|rate_limit|too many requests",
    r"(?i)overloaded|service unavailable|service_unavailable|service-unavailable|internal error|internal_error|internal server error|server error|server_error|server-error|provider returned error|provider_returned_error|provider-returned-error",
    r"(?i)terminated|fetch failed|failed to fetch|network error|upstream connect|connection error|connection refused|connection lost|socket connection was closed|socket hang up|reset before headers|getaddrinfo|enotfound|eai_again|econnrefused|econnreset|etimedout",
    r"(?i)^timeout$|\b(?:request|response|connection|network|stream|read) (?:timeout|timed out|time out)\b",
    r"(?i)try your request again|retry your request|resource exhausted|resource_exhausted",
];

fn matches_retryable_message(value: &str) -> bool {
    for pattern in RETRYABLE_PATTERNS {
        if let Ok(re) = regex::Regex::new(pattern) {
            if re.is_match(value) {
                return true;
            }
        }
    }
    false
}

fn cap(ms: u64) -> u64 {
    ms.min(RETRY_MAX_DELAY)
}

fn exponential(attempt: u32, random: f64) -> u64 {
    let base = RETRY_INITIAL_DELAY
        .saturating_mul(RETRY_BACKOFF_FACTOR.saturating_pow(attempt.saturating_sub(1)));
    (base as f64 + base as f64 * RETRY_JITTER_FACTOR * random).ceil() as u64
}

/// Calculate the retry delay for a given attempt and optional API error.
pub fn delay(attempt: u32, error: Option<&ApiErrorData>, random: f64) -> u64 {
    if let Some(err) = error {
        if let Some(headers) = &err.response_headers {
            if let Some(retry_after_ms) = headers.get("retry-after-ms") {
                if let Ok(parsed) = retry_after_ms.parse::<f64>() {
                    return cap(parsed as u64);
                }
            }

            if let Some(retry_after) = headers.get("retry-after") {
                if let Ok(parsed_seconds) = retry_after.parse::<f64>() {
                    return cap((parsed_seconds * 1000.0).ceil() as u64);
                }
                if let Ok(parsed_date) = chrono::DateTime::parse_from_rfc2822(retry_after) {
                    let now = chrono::Utc::now();
                    let diff = parsed_date.with_timezone(&chrono::Utc) - now;
                    let ms = diff.num_milliseconds();
                    if ms > 0 {
                        return cap(ms as u64);
                    }
                }
            }

            return cap(exponential(attempt, random));
        }
    }

    cap(exponential(attempt, random).min(RETRY_MAX_DELAY_NO_HEADERS))
}

/// Determine if an API error is retryable.
pub fn retryable(error: &ApiErrorData, provider: &str) -> Option<Retryable> {
    let status = error.status_code;

    if !error.is_retryable
        && !(status.is_some_and(|s| s >= 500))
        && !matches_retryable_message(&error.message)
        && !error
            .response_body
            .as_ref()
            .is_some_and(|b| matches_retryable_message(b))
    {
        return None;
    }

    if let Some(body) = &error.response_body {
        if body.contains("FreeUsageLimitError") {
            return Some(Retryable {
                message: GO_UPSELL_MESSAGE.to_string(),
                action: Some(RetryAction {
                    reason: "free_tier_limit".to_string(),
                    provider: provider.to_string(),
                    title: "Free limit reached".to_string(),
                    message: "Subscribe to OpenCode Go for reliable access to the best open-source models, starting at $5/month.".to_string(),
                    label: "subscribe".to_string(),
                    link: Some(GO_UPSELL_URL.to_string()),
                }),
            });
        }

        if body.contains("GoUsageLimitError") {
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(body) {
                let workspace = parsed
                    .pointer("/metadata/workspace")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let limit_name = parsed
                    .pointer("/metadata/limitName")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                let retry_after = error
                    .response_headers
                    .as_ref()
                    .and_then(|h| h.get("retry-after"))
                    .and_then(|v| v.parse::<f64>().ok());

                let reset_in = if let Some(ra) = retry_after {
                    let seconds = (ra.max(0.0).ceil()) as u64;
                    let days = seconds / 86_400;
                    let hours = (seconds % 86_400) / 3_600;
                    let minutes = ((seconds % 3_600) as f64 / 60.0).ceil() as u64;
                    let unit = |v: u64, n: &str| format!("{} {}{}", v, n, if v == 1 { "" } else { "s" });

                    if days > 0 {
                        if hours > 0 {
                            format!("{} {}", unit(days, "day"), unit(hours, "hour"))
                        } else {
                            unit(days, "day")
                        }
                    } else if hours > 0 {
                        if minutes > 0 {
                            format!("{} {}", unit(hours, "hour"), unit(minutes, "minute"))
                        } else {
                            unit(hours, "hour")
                        }
                    } else if minutes > 0 {
                        unit(minutes, "minute")
                    } else {
                        "less than a minute".to_string()
                    }
                } else {
                    String::new()
                };

                let message = format!(
                    "{} usage limit reached. It will reset in {}. To continue using this model now, enable usage from your available balance",
                    if limit_name.is_empty() { "Usage".to_string() } else { limit_name.to_string() },
                    reset_in
                );

                let link = format!("https://opencode.ai/workspace/{}/go", workspace);

                return Some(Retryable {
                    message: format!("{} - {}", message, link),
                    action: Some(RetryAction {
                        reason: "account_rate_limit".to_string(),
                        provider: provider.to_string(),
                        title: "Go limit reached".to_string(),
                        message,
                        label: "open settings".to_string(),
                        link: Some(link),
                    }),
                });
            }
        }
    }

    let msg = if error.message.contains("Overloaded") {
        "Provider is overloaded"
    } else {
        &error.message
    };
    Some(Retryable {
        message: msg.to_string(),
        action: None,
    })
}

/// Retry policy decision.
#[derive(Debug, Clone)]
pub struct RetryDecision {
    pub should_retry: bool,
    pub delay: Duration,
    pub attempt: u32,
}

/// Evaluate retry policy for a given attempt and error.
pub fn evaluate(
    attempt: u32,
    error: &ApiErrorData,
    provider: &str,
) -> Option<RetryDecision> {
    let _retry = retryable(error, provider)?;
    if attempt > RETRY_MAX_RETRIES {
        return None;
    }
    let ms = delay(attempt, Some(error), rand::random());
    Some(RetryDecision {
        should_retry: true,
        delay: Duration::from_millis(ms),
        attempt,
    })
}
