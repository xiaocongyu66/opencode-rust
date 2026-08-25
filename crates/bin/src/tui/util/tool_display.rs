use serde_json::Value;

pub fn web_search_provider_label(provider: &str) -> &'static str {
    match provider {
        "parallel" => "Parallel Web Search",
        "exa" => "Exa Web Search",
        _ => "Web Search",
    }
}

pub fn tool_display_metadata(state: &Value) -> Value {
    if state.is_null() || !state.is_object() || state.is_array() {
        return Value::Null;
    }
    let status = state.get("status");
    if status.is_none() || status == Some(&Value::String("pending".into())) {
        return Value::Null;
    }
    let structured = state.get("structured");
    match structured {
        Some(s) if s.is_object() && !s.is_array() => s.clone(),
        _ => Value::Null,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_web_search_labels() {
        assert_eq!(web_search_provider_label("parallel"), "Parallel Web Search");
        assert_eq!(web_search_provider_label("exa"), "Exa Web Search");
        assert_eq!(web_search_provider_label("other"), "Web Search");
    }

    #[test]
    fn test_metadata_pending() {
        let state = json!({"status": "pending", "structured": {"foo": "bar"}});
        assert_eq!(tool_display_metadata(&state), Value::Null);
    }

    #[test]
    fn test_metadata_valid() {
        let state = json!({"status": "completed", "structured": {"foo": "bar"}});
        assert_eq!(tool_display_metadata(&state), json!({"foo": "bar"}));
    }

    #[test]
    fn test_metadata_no_structured() {
        let state = json!({"status": "completed"});
        assert_eq!(tool_display_metadata(&state), Value::Null);
    }

    #[test]
    fn test_metadata_array() {
        let state = json!([1, 2, 3]);
        assert_eq!(tool_display_metadata(&state), Value::Null);
    }
}
