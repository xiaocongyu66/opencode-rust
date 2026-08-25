//! Grep tool — search file contents with regex.
//!
//! Aligned with claude-code-best GrepTool:
//! - `pattern` (required): regex pattern to search for
//! - `path` (optional): file or directory to search in
//! - `glob` (optional): glob pattern to filter files
//! - `output_mode` (optional): "content" | "files_with_matches" | "count"
//! - `-B` / `-A` / `-C` / `context` (optional): context lines

use async_trait::async_trait;
use serde::Deserialize;

use crate::tools::tool::{Tool, ToolContext, ToolFailure, ToolResult};

pub struct GrepTool;

impl GrepTool {
    pub fn new() -> Self {
        Self
    }
}

#[derive(Deserialize)]
struct GrepInput {
    pattern: String,
    #[serde(default)]
    path: Option<String>,
    #[serde(default)]
    glob: Option<String>,
    #[serde(default, rename = "output_mode")]
    output_mode: Option<String>,
    #[serde(default, rename = "-B")]
    before: Option<usize>,
    #[serde(default, rename = "-A")]
    after: Option<usize>,
    #[serde(default, rename = "-C")]
    context: Option<usize>,
}

const MAX_RESULTS: usize = 100;

#[async_trait]
impl Tool for GrepTool {
    fn name(&self) -> &str { "Grep"
    }

    fn description(&self) -> &str {
        "Searches file contents using a regular expression pattern. Returns \
         matching files, matching lines, or match counts depending on \
         output_mode. Defaults to files_with_matches."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "pattern": {
                    "type": "string",
                    "description": "The regular expression pattern to search for in file contents"
                },
                "path": {
                    "type": "string",
                    "description": "File or directory to search in. Defaults to current working directory."
                },
                "glob": {
                    "type": "string",
                    "description": "Glob pattern to filter files (e.g. \"*.js\", \"*.{ts,tsx}\")"
                },
                "output_mode": {
                    "type": "string",
                    "enum": ["content", "files_with_matches", "count"],
                    "description": "Output mode: \"content\" shows matching lines, \"files_with_matches\" shows file paths, \"count\" shows match counts. Defaults to \"files_with_matches\"."
                },
                "-B": {
                    "type": "number",
                    "description": "Number of lines to show before each match. Requires output_mode: \"content\"."
                },
                "-A": {
                    "type": "number",
                    "description": "Number of lines to show after each match. Requires output_mode: \"content\"."
                },
                "-C": {
                    "type": "number",
                    "description": "Alias for context."
                },
                "context": {
                    "type": "number",
                    "description": "Number of lines to show before and after each match. Requires output_mode: \"content\"."
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
        let input: GrepInput = serde_json::from_value(params)?;

        let re = regex::Regex::new(&input.pattern)
            .map_err(|e| ToolFailure::Message(format!("Invalid regex: {}", e)))?;

        let base = input.path.as_deref().unwrap_or(".");
        let mode = input.output_mode.as_deref().unwrap_or("files_with_matches");
        let context = input.context.or(input.context).or(input.before.or(input.after).map(|_| 0));
        let before = input.before.unwrap_or(context.unwrap_or(0));
        let after = input.after.unwrap_or(context.unwrap_or(0));

        let mut matches: Vec<FileMatch> = Vec::new();
        walk_grep(
            std::path::Path::new(base),
            &re,
            input.glob.as_deref(),
            &mut matches,
            0,
        )?;

        if matches.is_empty() {
            return Ok(ToolResult::text(format!(
                "No matches for '{}' in {}",
                input.pattern, base
            )));
        }

        let total_files = matches.len();
        matches.truncate(MAX_RESULTS);

        let result = match mode {
            "files_with_matches" => {
                let mut s = format!("Found {} files:\n", total_files);
                for m in &matches {
                    s.push_str(&m.path);
                    s.push('\n');
                }
                s
            }
            "count" => {
                let mut s = String::new();
                for m in &matches {
                    s.push_str(&format!("{}: {}\n", m.path, m.lines.len()));
                }
                s
            }
            _ => {
                // "content"
                let mut s = String::new();
                for m in &matches {
                    for line in &m.lines {
                        // Context before
                        for i in 1..=before.min(line.line_no - 1) {
                            let ctx_no = line.line_no - i;
                            if let Some(ctx) = line.context_lines.get(&(ctx_no)) {
                                s.push_str(&format!("{}-{}- {}\n", m.path, ctx_no, ctx));
                            }
                        }
                        s.push_str(&format!("{}:{}: {}\n", m.path, line.line_no, line.text));
                        // Context after
                        for i in 1..=after {
                            let ctx_no = line.line_no + i;
                            if let Some(ctx) = line.context_lines.get(&ctx_no) {
                                s.push_str(&format!("{}-{}- {}\n", m.path, ctx_no, ctx));
                            }
                        }
                    }
                }
                s
            }
        };

        Ok(ToolResult::text(result))
    }
}

struct LineMatch {
    line_no: usize,
    text: String,
    context_lines: std::collections::HashMap<usize, String>,
}

struct FileMatch {
    path: String,
    lines: Vec<LineMatch>,
}

fn walk_grep(
    dir: &std::path::Path,
    re: &regex::Regex,
    glob: Option<&str>,
    out: &mut Vec<FileMatch>,
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
        if path.is_dir() {
            // Skip hidden directories like .git
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.starts_with('.') {
                    continue;
                }
            }
            walk_grep(&path, re, glob, out, depth + 1)?;
        } else if path.is_file() {
            // Apply glob filter if present
            if let Some(g) = glob {
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if !simple_glob_match(g, name) {
                    continue;
                }
            }

            let content = match std::fs::read_to_string(&path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            let all_lines: Vec<&str> = content.lines().collect();
            let mut file_matches: Vec<LineMatch> = Vec::new();

            for (i, line) in all_lines.iter().enumerate() {
                if re.is_match(line) {
                    let mut context = std::collections::HashMap::new();
                    // Collect context lines (a window around the match)
                    let start = i.saturating_sub(5);
                    let end = (i + 6).min(all_lines.len());
                    for j in start..end {
                        context.insert(j + 1, all_lines[j].to_string());
                    }
                    file_matches.push(LineMatch {
                        line_no: i + 1,
                        text: line.to_string(),
                        context_lines: context,
                    });
                }
            }

            if !file_matches.is_empty() {
                out.push(FileMatch {
                    path: path.to_string_lossy().to_string(),
                    lines: file_matches,
                });
            }
        }
    }
    Ok(())
}

fn simple_glob_match(pattern: &str, text: &str) -> bool {
    // Simple: support * and exact match
    if pattern == text {
        return true;
    }
    if !pattern.contains('*') {
        return false;
    }
    // Very basic: *.{ts,tsx} → check extension
    if let Some(exts) = pattern.strip_prefix("*.{").and_then(|p| p.strip_suffix("}")) {
        for ext in exts.split(',') {
            if text.ends_with(&format!(".{}", ext)) {
                return true;
            }
        }
        return false;
    }
    if let Some(ext) = pattern.strip_prefix("*.") {
        return text.ends_with(&format!(".{}", ext));
    }
    // Fallback: use glob_match from glob.rs logic
    crate::tools::glob::glob_match_internal(pattern, text)
}
