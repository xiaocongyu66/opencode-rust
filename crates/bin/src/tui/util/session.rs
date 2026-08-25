use regex::Regex;
use std::sync::LazyLock;

static DEFAULT_TITLE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(New session - |Child session - )\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{3}Z$").unwrap()
});

pub fn is_default_title(title: &str) -> bool {
    DEFAULT_TITLE_RE.is_match(title)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_title() {
        assert!(is_default_title("New session - 2024-01-15T10:30:00.000Z"));
        assert!(is_default_title("Child session - 2024-01-15T10:30:00.000Z"));
    }

    #[test]
    fn test_custom_title() {
        assert!(!is_default_title("My custom title"));
        assert!(!is_default_title("New session - invalid"));
    }
}
