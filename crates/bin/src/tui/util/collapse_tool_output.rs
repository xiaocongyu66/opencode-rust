#[derive(Debug, Clone)]
pub struct CollapseResult {
    pub output: String,
    pub overflow: bool,
}

pub fn collapse_tool_output(output: &str, max_lines: usize, max_chars: usize) -> CollapseResult {
    let lines: Vec<&str> = output.split('\n').collect();
    let total_chars = output.chars().count();

    if lines.len() <= max_lines && total_chars <= max_chars {
        return CollapseResult {
            output: output.to_string(),
            overflow: false,
        };
    }

    let preview: String = lines
        .iter()
        .take(max_lines)
        .copied()
        .collect::<Vec<_>>()
        .join("\n");

    if preview.chars().count() > max_chars {
        let truncated: String = preview.chars().take(max_chars.saturating_sub(1)).collect();
        return CollapseResult {
            output: format!("{}…", truncated),
            overflow: true,
        };
    }

    let mut result_lines: Vec<String> = lines.iter().take(max_lines).map(|s| s.to_string()).collect();
    result_lines.push("…".to_string());
    CollapseResult {
        output: result_lines.join("\n"),
        overflow: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_collapse() {
        let result = collapse_tool_output("hello\nworld", 10, 100);
        assert!(!result.overflow);
        assert_eq!(result.output, "hello\nworld");
    }

    #[test]
    fn test_too_many_lines() {
        let output = "line1\nline2\nline3\nline4\nline5";
        let result = collapse_tool_output(output, 3, 100);
        assert!(result.overflow);
        assert!(result.output.contains("…"));
    }

    #[test]
    fn test_too_many_chars() {
        let output = "abcdefghijklmnopqrstuvwxyz";
        let result = collapse_tool_output(output, 10, 10);
        assert!(result.overflow);
        assert!(result.output.ends_with('…'));
        assert!(result.output.chars().count() <= 10);
    }

    #[test]
    fn test_preview_char_limit() {
        let output = "aaaaaaaaaaaaaaaa\nbbbbbbbbbbbbbbbb";
        let result = collapse_tool_output(output, 1, 5);
        assert!(result.overflow);
        assert!(result.output.ends_with('…'));
    }

    #[test]
    fn test_single_line_no_overflow() {
        let result = collapse_tool_output("short", 10, 100);
        assert!(!result.overflow);
    }
}
