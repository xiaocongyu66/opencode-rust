use serde_json::Value;

pub fn is_record(value: &Value) -> bool {
    value.is_object() && !value.is_array()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_object() {
        assert!(is_record(&json!({"a": 1})));
    }

    #[test]
    fn test_array() {
        assert!(!is_record(&json!([1, 2, 3])));
    }

    #[test]
    fn test_null() {
        assert!(!is_record(&Value::Null));
    }

    #[test]
    fn test_string() {
        assert!(!is_record(&json!("hello")));
    }
}
