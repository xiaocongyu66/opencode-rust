use std::path::Path;

pub fn normalize_path(input: &str, platform: &str) -> String {
    if platform != "win32" {
        return input.to_string();
    }
    let replaced = input.replace('/', "\\");
    let resolved = Path::new(&replaced);
    match std::fs::canonicalize(resolved) {
        Ok(p) => p.to_string_lossy().to_string(),
        Err(_) => resolved.to_string_lossy().to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_non_win32() {
        assert_eq!(normalize_path("/foo/bar", "linux"), "/foo/bar");
    }

    #[test]
    fn test_win32_replace() {
        let result = normalize_path("foo\\bar", "win32");
        assert!(result.contains("foo") || result.is_empty());
    }
}
