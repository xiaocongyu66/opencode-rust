use std::collections::HashSet;

pub fn is_console_managed_provider(
    console_managed: &HashSet<String>,
    provider_id: &str,
) -> bool {
    console_managed.contains(provider_id)
}

pub fn is_console_managed_provider_slice(
    console_managed: &[String],
    provider_id: &str,
) -> bool {
    console_managed.iter().any(|p| p == provider_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set() {
        let mut set = HashSet::new();
        set.insert("openai".to_string());
        assert!(is_console_managed_provider(&set, "openai"));
        assert!(!is_console_managed_provider(&set, "anthropic"));
    }

    #[test]
    fn test_slice() {
        let list = vec!["openai".to_string(), "anthropic".to_string()];
        assert!(is_console_managed_provider_slice(&list, "openai"));
        assert!(!is_console_managed_provider_slice(&list, "google"));
    }
}
