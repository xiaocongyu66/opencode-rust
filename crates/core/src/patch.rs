//! Patch utilities — diff and patch operations.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Patch {
    pub file: String,
    pub hunks: Vec<Hunk>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Hunk {
    pub old_start: usize,
    pub old_lines: usize,
    pub new_start: usize,
    pub new_lines: usize,
    pub lines: Vec<String>,
}

pub fn apply_patch(content: &str, patch: &Patch) -> Result<String, String> {
    let mut lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();

    for hunk in &patch.hunks {
        let start = hunk.new_start.saturating_sub(1);
        let new_lines: Vec<String> = hunk.lines.iter()
            .filter(|l| !l.starts_with('-'))
            .map(|l| l.trim_start_matches('+').to_string())
            .collect();

        let end = start + hunk.old_lines.min(lines.len() - start);
        lines.splice(start..end, new_lines);
    }

    Ok(lines.join("\n"))
}
