use serde_json::Value;

pub fn is_record(value: &Value) -> bool {
    value.is_object() && !value.is_array()
}

fn tagged(value: &Value, tag: &str) -> bool {
    is_record(value) && value.get("_tag").and_then(|v| v.as_str()) == Some(tag)
}

fn named(value: &Value, name: &str) -> bool {
    is_record(value)
        && (value.get("name").and_then(|v| v.as_str()) == Some(name)
            || value.get("_tag").and_then(|v| v.as_str()) == Some(name))
}

fn config_data(value: &Value, tag: &str) -> Option<Value> {
    if !is_record(value) {
        return None;
    }
    if value.get("name").and_then(|v| v.as_str()) == Some(tag) {
        return value.get("data").cloned();
    }
    if value.get("_tag").and_then(|v| v.as_str()) == Some(tag) {
        return Some(value.clone());
    }
    None
}

fn field(value: &Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

pub fn cli_error_message(input: &Value) -> Option<String> {
    if tagged(input, "CliError") {
        return Some(field(input, "message").unwrap_or_default());
    }
    if tagged(input, "AccountServiceError") || tagged(input, "AccountTransportError") {
        return Some(field(input, "message").unwrap_or_default());
    }

    if let Some(model) = config_data(input, "ProviderModelNotFoundError") {
        let provider_id = field(&model, "providerID").unwrap_or_default();
        let model_id = field(&model, "modelID").unwrap_or_default();
        let suggestions: Vec<String> = model
            .get("suggestions")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();
        let mut parts = vec![format!("Model not found: {}/{}", provider_id, model_id)];
        if !suggestions.is_empty() {
            parts.push(format!("Did you mean: {}", suggestions.join(", ")));
        }
        parts.push("Try: `opencode models` to list available models".to_string());
        parts.push("Or check your config (opencode.json) provider/model names".to_string());
        return Some(parts.join("\n"));
    }

    if let Some(provider) = config_data(input, "ProviderInitError") {
        let provider_id = field(&provider, "providerID").unwrap_or_default();
        return Some(format!(
            "Failed to initialize provider \"{}\". Check credentials and configuration.",
            provider_id
        ));
    }

    if let Some(json_err) = config_data(input, "ConfigJsonError") {
        let path = field(&json_err, "path").unwrap_or_default();
        let message = field(&json_err, "message");
        return Some(format!(
            "Config file at {} is not valid JSON(C){}",
            path,
            message.map(|m| format!(": {}", m)).unwrap_or_default()
        ));
    }

    if let Some(dir) = config_data(input, "ConfigDirectoryTypoError") {
        let dir_name = field(&dir, "dir").unwrap_or_default();
        let path = field(&dir, "path").unwrap_or_default();
        let suggestion = field(&dir, "suggestion").unwrap_or_default();
        return Some(format!(
            "Directory \"{}\" in {} is not valid. Rename the directory to \"{}\" or remove it. This is a common typo.",
            dir_name, path, suggestion
        ));
    }

    if let Some(frontmatter) = config_data(input, "ConfigFrontmatterError") {
        return Some(field(&frontmatter, "message").unwrap_or_default());
    }

    if let Some(invalid) = config_data(input, "ConfigInvalidError") {
        let path = field(&invalid, "path");
        let message = field(&invalid, "message");
        let issues: Vec<String> = invalid
            .get("issues")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|issue| {
                        let msg = issue.get("message")?.as_str()?.to_string();
                        let path_arr = issue.get("path")?.as_array()?;
                        let path_parts: Vec<String> = path_arr
                            .iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect();
                        Some(format!("↳ {} {}", msg, path_parts.join(".")))
                    })
                    .collect()
            })
            .unwrap_or_default();
        let header = format!(
            "Configuration is invalid{}{}",
            path.as_ref()
                .filter(|p| p.as_str() != "config")
                .map(|p| format!(" at {}", p))
                .unwrap_or_default(),
            message.map(|m| format!(": {}", m)).unwrap_or_default()
        );
        let mut parts = vec![header];
        parts.extend(issues);
        return Some(parts.join("\n"));
    }

    if tagged(input, "UICancelledError") || named(input, "UICancelledError") {
        return Some(String::new());
    }

    if is_record(input) && named(input, "MCPFailed") {
        let name = input
            .get("data")
            .and_then(|d| field(d, "name"))
            .unwrap_or_default();
        return Some(format!(
            "MCP server \"{}\" failed. Note, opencode does not support MCP authentication yet.",
            name
        ));
    }

    None
}

pub fn error_format(error: &Value) -> String {
    if error.is_null() {
        return "null".to_string();
    }
    match error {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        _ => {
            let json = serde_json::to_string_pretty(error).unwrap_or_else(|_| "{}".to_string());
            if json == "{}" {
                let keys: Vec<&String> = error.as_object().map(|m| m.keys().collect()).unwrap_or_default();
                if keys.is_empty() {
                    "Error (no message)".to_string()
                } else {
                    format!("Error {{ {} }}", keys.iter().map(|k| k.as_str()).collect::<Vec<_>>().join(", "))
                }
            } else {
                json
            }
        }
    }
}

pub fn error_message(error: &Value) -> String {
    if let Some(msg) = error.get("message").and_then(|v| v.as_str()) {
        if !msg.is_empty() {
            return msg.to_string();
        }
    }
    if let Some(data) = error.get("data") {
        if let Some(msg) = data.get("message").and_then(|v| v.as_str()) {
            if !msg.is_empty() {
                return msg.to_string();
            }
        }
    }
    match error {
        Value::String(s) if !s.is_empty() => s.clone(),
        _ => {
            let formatted = error_format(error);
            if !formatted.is_empty() && formatted != "null" {
                formatted
            } else {
                "unknown error".to_string()
            }
        }
    }
}

pub fn error_data(error: &Value) -> Value {
    let mut data = if is_record(error) {
        let mut obj = serde_json::Map::new();
        if let Some(map) = error.as_object() {
            for (key, value) in map {
                match value {
                    Value::String(_) | Value::Number(_) | Value::Bool(_) => {
                        obj.insert(key.clone(), value.clone());
                    }
                    Value::Null => {}
                    _ => {
                        obj.insert(key.clone(), Value::String(value.to_string()));
                    }
                }
            }
        }
        Value::Object(obj)
    } else {
        let mut obj = serde_json::Map::new();
        obj.insert(
            "type".to_string(),
            Value::String(error_type_string(error)),
        );
        obj.insert("message".to_string(), Value::String(error_message(error)));
        obj.insert(
            "formatted".to_string(),
            Value::String(error_format(error)),
        );
        Value::Object(obj)
    };

    if let Some(obj) = data.as_object_mut() {
        if !obj.contains_key("message") || obj.get("message").and_then(|v| v.as_str()) == Some("") {
            obj.insert("message".to_string(), Value::String(error_message(error)));
        }
        if !obj.contains_key("type") {
            obj.insert("type".to_string(), Value::String(error_type_string(error)));
        }
        if !obj.contains_key("formatted") {
            obj.insert("formatted".to_string(), Value::String(error_format(error)));
        }
    }
    data
}

fn error_type_string(error: &Value) -> String {
    match error {
        Value::Null => "null".to_string(),
        Value::Bool(_) => "boolean".to_string(),
        Value::Number(_) => "number".to_string(),
        Value::String(_) => "string".to_string(),
        Value::Array(_) => "array".to_string(),
        Value::Object(_) => "object".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_cli_error_tagged() {
        let input = json!({"_tag": "CliError", "message": "Something went wrong"});
        assert_eq!(
            cli_error_message(&input),
            Some("Something went wrong".to_string())
        );
    }

    #[test]
    fn test_model_not_found() {
        let input = json!({
            "name": "ProviderModelNotFoundError",
            "data": {
                "providerID": "openai",
                "modelID": "gpt-5",
                "suggestions": ["gpt-4", "gpt-3.5"]
            }
        });
        let result = cli_error_message(&input).unwrap();
        assert!(result.contains("Model not found: openai/gpt-5"));
        assert!(result.contains("Did you mean: gpt-4, gpt-3.5"));
    }

    #[test]
    fn test_ui_cancelled() {
        let input = json!({"_tag": "UICancelledError"});
        assert_eq!(cli_error_message(&input), Some(String::new()));
    }

    #[test]
    fn test_error_message_from_message() {
        let input = json!({"message": "test error"});
        assert_eq!(error_message(&input), "test error");
    }

    #[test]
    fn test_error_message_from_data() {
        let input = json!({"data": {"message": "nested error"}});
        assert_eq!(error_message(&input), "nested error");
    }

    #[test]
    fn test_error_format_string() {
        assert_eq!(error_format(&json!("hello")), "hello");
    }

    #[test]
    fn test_error_format_object() {
        let result = error_format(&json!({"a": 1}));
        assert!(result.contains("\"a\""));
        assert!(result.contains("1"));
    }
}
