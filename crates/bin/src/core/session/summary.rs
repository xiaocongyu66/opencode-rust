//! Session summary management.
//!
//! Ported from `session/summary.ts`.
//! Computes file diffs and manages session summaries.


use crate::schema::session::SessionMessage;

/// Session summary info.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct SessionSummary {
    pub additions: u64,
    pub deletions: u64,
    pub files: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diffs: Option<Vec<FileDiff>>,
}

/// File diff info.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct FileDiff {
    pub file: String,
    pub additions: u64,
    pub deletions: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub binary: Option<bool>,
}

/// Unquote a git-quoted path.
pub fn unquote_git_path(input: &str) -> String {
    if !input.starts_with('"') || !input.ends_with('"') {
        return input.to_string();
    }

    let body = &input[1..input.len() - 1];
    let mut result = Vec::new();
    let chars: Vec<char> = body.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];
        if c != '\\' {
            result.push(c as u8);
            i += 1;
            continue;
        }

        i += 1;
        if i >= chars.len() {
            result.push(b'\\');
            break;
        }

        let next = chars[i];
        if next.is_ascii_digit() && next <= '7' {
            let chunk: String = chars[i..].iter().take(3).collect();
            if let Ok(parsed) = u8::from_str_radix(&chunk, 8) {
                result.push(parsed);
                i += chunk.len();
                continue;
            }
        }

        let escaped = match next {
            'n' => '\n',
            'r' => '\r',
            't' => '\t',
            'b' => '\x08',
            'f' => '\x0c',
            'v' => '\x0b',
            '\\' | '"' => next,
            _ => next,
        };
        result.push(escaped as u8);
        i += 1;
    }

    String::from_utf8_lossy(&result).to_string()
}

/// Compute diffs from message snapshots.
pub fn compute_diff(messages: &[SessionMessage]) -> Vec<FileDiff> {
    let _from: Option<String> = None;
    let _to: Option<String> = None;

    for msg in messages {
        if let SessionMessage::Assistant { content, .. } = msg {
            for item in content {
                if let crate::schema::session::AssistantContent::Tool { .. } = item {
                    // Track snapshots from step-start/step-finish parts
                    // In the full implementation this would use snapshot hashes
                }
            }
        }
    }

    Vec::new()
}

/// Summarize a session turn.
pub fn summarize(
    _session_id: &str,
    _message_id: &str,
    messages: &[SessionMessage],
) -> SessionSummary {
    let diffs = compute_diff(messages);
    SessionSummary {
        additions: diffs.iter().map(|d| d.additions).sum(),
        deletions: diffs.iter().map(|d| d.deletions).sum(),
        files: diffs.len() as u64,
        diffs: if diffs.is_empty() { None } else { Some(diffs) },
    }
}
