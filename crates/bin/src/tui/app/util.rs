//! Small utility helpers for the TUI app module: debug logging,
//! semver comparison, and error-message extraction.

/// Debug log: writes to `~/.rsopencode/debug.log` (rotated on each launch).
/// Used during development to trace execution flow without interfering
/// with the TUI's terminal output.
pub fn dbg_log(msg: &str) {
    use std::io::Write;
    if let Some(home) = dirs::home_dir() {
        let path = home.join(".rsopencode").join("debug.log");
        if let Ok(mut f) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
        {
            let _ = writeln!(f, "{} {}", chrono::Local::now().format("%H:%M:%S"), msg);
        }
    }
}

/// Parse a version string like "1.2.3" or "v1.2.3-beta" into
/// (core parts, optional prerelease). Invalid numeric parts become 0
/// so the vector length matches the number of dot-separated segments.
pub fn parse_version(value: &str) -> (Vec<u32>, Option<String>) {
    let cleaned = value.strip_prefix('v').unwrap_or(value);
    let (core, prerelease) = match cleaned.split_once('-') {
        Some((c, p)) => (c, Some(p.to_string())),
        None => (cleaned, None),
    };
    let parts: Vec<u32> = core
        .split('.')
        .map(|s| s.parse::<u32>().unwrap_or(0))
        .collect();
    (parts, prerelease)
}

/// Returns true if `left` is a newer version than `right`.
/// Prerelease versions are considered older than the same version
/// without a prerelease tag.
pub fn is_version_greater(left: &str, right: &str) -> bool {
    let (l_parts, l_pre) = parse_version(left);
    let (r_parts, r_pre) = parse_version(right);
    let len = l_parts.len().max(r_parts.len());
    for i in 0..len {
        let l = l_parts.get(i).copied().unwrap_or(0);
        let r = r_parts.get(i).copied().unwrap_or(0);
        if l != r {
            return l > r;
        }
    }
    // Core versions equal — prerelease comparison.
    // No prerelease > has prerelease (1.0.0 > 1.0.0-beta).
    match (&l_pre, &r_pre) {
        (None, Some(_)) => true,
        (Some(_), None) => false,
        (Some(a), Some(b)) => a > b,
        (None, None) => false,
    }
}

/// Extract a human-readable error message from an error JSON value.
/// Checks `data.message`, then `message`, then the MessageAbortedError
/// name, and finally falls back to the raw JSON string.
pub fn error_message(error: &serde_json::Value) -> String {
    // `{"data": {"message": "..."}}` — nested error envelope.
    if let Some(data) = error.get("data") {
        if let Some(msg) = data.get("message").and_then(|v| v.as_str()) {
            return msg.to_string();
        }
    }
    // `{"message": "..."}` — flat error object.
    if let Some(msg) = error.get("message").and_then(|v| v.as_str()) {
        return msg.to_string();
    }
    // `{"name": "MessageAbortedError"}` — user-interrupted tool/message.
    if let Some(name) = error.get("name").and_then(|v| v.as_str()) {
        if name == "MessageAbortedError" {
            return "interrupted".to_string();
        }
    }
    if let Some(s) = error.as_str() {
        return s.to_string();
    }
    error.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_version_greater_basic() {
        assert!(is_version_greater("1.0.1", "1.0.0"));
        assert!(!is_version_greater("1.0.0", "1.0.1"));
        assert!(!is_version_greater("1.0.0", "1.0.0"));
    }

    #[test]
    fn test_is_version_greater_major() {
        assert!(is_version_greater("2.0.0", "1.9.9"));
        assert!(!is_version_greater("1.9.9", "2.0.0"));
    }

    #[test]
    fn test_is_version_greater_v_prefix() {
        assert!(is_version_greater("v1.0.1", "1.0.0"));
        assert!(is_version_greater("1.0.1", "v1.0.0"));
    }

    #[test]
    fn test_is_version_greater_prerelease() {
        // 1.0.0 > 1.0.0-beta
        assert!(is_version_greater("1.0.0", "1.0.0-beta"));
        assert!(!is_version_greater("1.0.0-beta", "1.0.0"));
    }

    #[test]
    fn test_is_version_greater_different_lengths() {
        assert!(is_version_greater("1.0.0.1", "1.0.0"));
        assert!(!is_version_greater("1.0.0", "1.0.0.1"));
    }

    #[test]
    fn test_parse_version_simple() {
        assert_eq!(parse_version("1.2.3"), (vec![1, 2, 3], None));
    }

    #[test]
    fn test_parse_version_v_prefix() {
        assert_eq!(parse_version("v1.2.3"), (vec![1, 2, 3], None));
    }

    #[test]
    fn test_parse_version_prerelease() {
        let (parts, pre) = parse_version("1.2.3-beta.1");
        assert_eq!(parts, vec![1, 2, 3]);
        assert_eq!(pre, Some("beta.1".to_string()));
    }

    #[test]
    fn test_parse_version_invalid_parts() {
        // Invalid numeric segments become 0 (placeholder for the position).
        let (parts, _) = parse_version("1.x.3");
        assert_eq!(parts, vec![1, 0, 3]);
    }
}
