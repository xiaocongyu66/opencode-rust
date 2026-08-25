//! Glob tool — find files matching a pattern.
//!
//! Aligned with claude-code-best GlobTool:
//! - `pattern` (required): glob pattern to match files against
//! - `path` (optional): directory to search in (defaults to cwd)

use async_trait::async_trait;
use serde::Deserialize;

use crate::tools::tool::{Tool, ToolContext, ToolFailure, ToolResult};

pub struct GlobTool;

impl GlobTool {
    pub fn new() -> Self {
        Self
    }
}

#[derive(Deserialize)]
struct GlobInput {
    pattern: String,
    #[serde(default)]
    path: Option<String>,
}

const MAX_RESULTS: usize = 100;

#[async_trait]
impl Tool for GlobTool {
    fn name(&self) -> &str { "Glob"
    }

    fn description(&self) -> &str {
        "Finds files matching a glob pattern. Recursively searches the \
         given directory (or current working directory). Supports patterns \
         like `**/*.rs`, `src/**/*.ts`."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "pattern": {
                    "type": "string",
                    "description": "The glob pattern to match files against"
                },
                "path": {
                    "type": "string",
                    "description": "The directory to search in. If not specified, the current working directory will be used."
                }
            },
            "required": ["pattern"]
        })
    }

    async fn execute(
        &self,
        params: serde_json::Value,
        _ctx: &ToolContext,
    ) -> Result<ToolResult, ToolFailure> {
        let input: GlobInput = serde_json::from_value(params)?;

        let base = input.path.as_deref().unwrap_or(".");
        let pattern = &input.pattern;

        let mut matches: Vec<String> = Vec::new();
        walk_glob(std::path::Path::new(base), pattern, &mut matches, 0)?;

        matches.sort();
        matches.truncate(MAX_RESULTS);

        if matches.is_empty() {
            Ok(ToolResult::text(format!(
                "No files matching '{}' in {}",
                pattern, base
            )))
        } else {
            let mut result = format!("Found {} files:\n", matches.len());
            for m in &matches {
                result.push_str(m);
                result.push('\n');
            }
            Ok(ToolResult::text(result))
        }
    }
}

/// Recursively walk the directory tree, matching files against the glob pattern.
fn walk_glob(
    dir: &std::path::Path,
    pattern: &str,
    out: &mut Vec<String>,
    depth: u32,
) -> Result<(), ToolFailure> {
    if depth > 20 || out.len() >= MAX_RESULTS * 2 {
        return Ok(());
    }

    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return Ok(()),
    };

    for entry in entries.flatten() {
        if out.len() >= MAX_RESULTS * 2 {
            return Ok(());
        }
        let path = entry.path();
        let rel = path.strip_prefix(dir).unwrap_or(&path);
        let rel_str = rel.to_string_lossy();

        if path.is_dir() {
            // Check if the directory itself matches the pattern (rare but possible).
            walk_glob(&path, pattern, out, depth + 1)?;
        } else if glob_match_internal(pattern, &rel_str) {
            out.push(path.to_string_lossy().to_string());
        }
    }
    Ok(())
}

/// Simple glob matcher supporting `*`, `**`, and `?`.
/// Public so other tools (e.g. grep) can reuse it.
pub fn glob_match_internal(pattern: &str, text: &str) -> bool {
    glob_match_segments(pattern, text)
}

/// Match using a simplified glob algorithm.
fn glob_match_segments(pattern: &str, text: &str) -> bool {
    // Normalize: treat ** as multi-segment wildcard.
    let p: Vec<char> = pattern.chars().collect();
    let t: Vec<char> = text.chars().collect();
    match_helper(&p, 0, &t, 0)
}

fn match_helper(p: &[char], pi: usize, t: &[char], ti: usize) -> bool {
    if pi == p.len() {
        return ti == t.len();
    }

    // Check for **
    if pi + 1 < p.len() && p[pi] == '*' && p[pi + 1] == '*' {
        // Skip trailing slashes after **
        let mut next_pi = pi + 2;
        while next_pi < p.len() && (p[next_pi] == '/' || p[next_pi] == '\\') {
            next_pi += 1;
        }
        // ** matches zero or more path segments
        for i in ti..=t.len() {
            if match_helper(p, next_pi, t, i) {
                return true;
            }
        }
        return false;
    }

    if p[pi] == '*' {
        // Single * matches within a segment (not across /)
        let mut next_ti = ti;
        while next_ti < t.len() && t[next_ti] != '/' && t[next_ti] != '\\' {
            if match_helper(p, pi + 1, t, next_ti) {
                return true;
            }
            next_ti += 1;
        }
        return match_helper(p, pi + 1, t, ti) || match_helper(p, pi + 1, t, next_ti);
    }

    if p[pi] == '?' {
        return ti < t.len() && t[ti] != '/' && match_helper(p, pi + 1, t, ti + 1);
    }

    if ti < t.len() && p[pi] == t[ti] {
        return match_helper(p, pi + 1, t, ti + 1);
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn glob_matches_simple() {
        assert!(glob_match_internal("*.rs", "main.rs"));
        assert!(!glob_match_internal("*.rs", "main.ts"));
    }

    #[test]
    fn glob_matches_double_star() {
        assert!(glob_match_internal("**/*.rs", "src/main.rs"));
        assert!(glob_match_internal("**/*.rs", "a/b/c/d.rs"));
    }

    #[test]
    fn glob_matches_question_mark() {
        assert!(glob_match_internal("?.rs", "a.rs"));
        assert!(!glob_match_internal("?.rs", "ab.rs"));
    }
}
