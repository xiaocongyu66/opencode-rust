use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct Args {
    pub model: Option<String>,
    pub agent: Option<String>,
    pub prompt: Option<String>,
    pub continue_session: Option<bool>,
    pub session_id: Option<String>,
    pub fork: Option<bool>,
    pub auto: Option<bool>,
}

impl Args {
    pub fn from_env_and_cli(cli: HashMap<String, String>) -> Self {
        let get = |key: &str| cli.get(key).cloned();
        Self {
            model: get("model"),
            agent: get("agent"),
            prompt: get("prompt"),
            continue_session: cli.get("continue").map(|v| v == "true" || v == "1"),
            session_id: get("session-id").or_else(|| get("sessionID")),
            fork: cli.get("fork").map(|v| v == "true" || v == "1"),
            auto: cli.get("auto").map(|v| v == "true" || v == "1"),
        }
    }
}
