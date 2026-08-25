use super::kv::KvContext;
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq)]
pub enum ThinkingMode {
    Show,
    Hide,
}

const MODES: &[ThinkingMode] = &[ThinkingMode::Show, ThinkingMode::Hide];

pub fn is_thinking_mode(value: &str) -> bool {
    matches!(value, "show" | "hide")
}

pub fn parse_thinking_mode(value: &str) -> ThinkingMode {
    match value {
        "show" => ThinkingMode::Show,
        _ => ThinkingMode::Hide,
    }
}

pub fn next_thinking_mode(current: &ThinkingMode) -> ThinkingMode {
    let idx = MODES.iter().position(|m| m == current).unwrap_or(0);
    MODES[(idx + 1) % MODES.len()].clone()
}

pub struct ReasoningSummary {
    pub title: Option<String>,
    pub body: String,
}

pub fn reasoning_summary(text: &str) -> ReasoningSummary {
    let content = text.trim();
    if let Some(rest) = content.strip_prefix("**") {
        if let Some(end) = rest.find("**") {
            let title = rest[..end].trim().to_string();
            let after = &rest[end + 2..];
            let body = after.trim_start_matches(['\r', '\n']).trim_end().to_string();
            return ReasoningSummary {
                title: Some(title),
                body,
            };
        }
    }
    ReasoningSummary {
        title: None,
        body: content.to_string(),
    }
}

pub struct ThinkingModeContext {
    kv: Arc<KvContext>,
}

impl ThinkingModeContext {
    pub fn new(kv: Arc<KvContext>) -> Self {
        Self { kv }
    }

    pub fn mode(&self) -> ThinkingMode {
        let stored = self.kv.get("thinking_mode");
        let value = stored
            .as_ref()
            .and_then(|v| v.as_str())
            .unwrap_or("hide");
        if is_thinking_mode(value) {
            parse_thinking_mode(value)
        } else {
            if !self.kv.get("thinking_mode").is_some() {
                let legacy = self.kv.get("thinking_visibility").and_then(|v| v.as_bool());
                match legacy {
                    Some(true) => {
                        self.kv.set("thinking_mode", serde_json::Value::String("show".to_string()));
                        return ThinkingMode::Show;
                    }
                    Some(false) => {
                        self.kv.set("thinking_mode", serde_json::Value::String("hide".to_string()));
                        return ThinkingMode::Hide;
                    }
                    None => {
                        self.kv.set("thinking_mode", serde_json::Value::String("hide".to_string()));
                        return ThinkingMode::Hide;
                    }
                }
            }
            ThinkingMode::Hide
        }
    }

    pub fn set(&self, mode: ThinkingMode) {
        let value = match mode {
            ThinkingMode::Show => "show",
            ThinkingMode::Hide => "hide",
        };
        self.kv.set(
            "thinking_mode",
            serde_json::Value::String(value.to_string()),
        );
    }

    pub fn toggle(&self) {
        let current = self.mode();
        self.set(next_thinking_mode(&current));
    }
}
