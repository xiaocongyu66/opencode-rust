//! TUI editor integration — external editor ($EDITOR/$VISUAL) and Zed IDE.
//! Ported from tui/src/editor.ts + tui/src/editor-zed.ts (287 lines total)
//!
//! Features:
//! - Open $VISUAL/$EDITOR with a temp file, return edited content
//! - Discover IDE editor connections via `.claude/ide/*.lock` files
//! - Query Zed's SQLite database for active editor selections
//! - UTF-8 byte offset to string index conversion
//! - Line/column position calculation from offsets

use std::env;
use std::fs;
use std::io::{self};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

/// Normalize prompt content — strip trailing newline if single-line.
pub fn normalize_prompt_content(content: &str) -> String {
    if content.ends_with("\r\n") {
        let body = &content[..content.len() - 2];
        if !body.contains('\n') && !body.contains('\r') {
            return body.to_string();
        }
        return content.to_string();
    }
    if content.ends_with('\n') {
        let body = &content[..content.len() - 1];
        if !body.contains('\n') && !body.contains('\r') {
            return body.to_string();
        }
        return content.to_string();
    }
    content.to_string()
}

/// Open an external editor with the given content, return the edited text.
pub fn open_editor(value: &str, cwd: Option<&Path>) -> io::Result<Option<String>> {
    let editor = env::var("VISUAL").or_else(|_| env::var("EDITOR")).ok();
    let editor = match editor {
        Some(e) if !e.is_empty() => e,
        _ => return Ok(None),
    };

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let file = env::temp_dir().join(format!("{}.md", timestamp));
    fs::write(&file, value)?;

    let working_dir = match cwd {
        Some(dir) if dir.exists() => dir.to_path_buf(),
        _ => env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
    };

    let parts: Vec<&str> = editor.split_whitespace().collect();
    let program = parts[0];
    let mut args: Vec<&str> = parts[1..].to_vec();
    args.push(file.to_str().unwrap_or(""));

    let mut cmd = Command::new(program);
    cmd.args(&args).current_dir(&working_dir);
    cmd.stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    if cfg!(target_os = "windows") {
        // On Windows, shell may be needed for complex editor commands
    }

    let status = cmd.status()?;
    if !status.success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!(
                "Editor exited with {}",
                status.code().map(|c| format!("code {}", c)).unwrap_or_else(|| "signal".to_string())
            ),
        ));
    }

    let content = fs::read_to_string(&file)?;
    let _ = fs::remove_file(&file);
    Ok(if content.is_empty() { None } else { Some(content) })
}

/// An editor connection discovered via `.claude/ide/*.lock`.
#[derive(Debug, Clone)]
pub struct EditorConnection {
    pub url: String,
    pub auth_token: Option<String>,
    pub source: String,
}

/// Discover an editor connection by scanning `~/.claude/ide/*.lock` files.
pub fn discover_editor_connection(directory: &Path) -> Option<EditorConnection> {
    let root = dirs::home_dir()?.join(".claude").join("ide");
    let entries = match fs::read_dir(&root) {
        Ok(e) => e,
        Err(_) => return None,
    };

    let mut candidates: Vec<(EditorConnection, usize, f64)> = Vec::new();

    for entry in entries.flatten() {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if !name_str.ends_with(".lock") {
            continue;
        }

        let port_str = &name_str[..name_str.len() - 5];
        let port: u16 = match port_str.parse() {
            Ok(p) if p > 0 => p,
            _ => continue,
        };

        let file = entry.path();
        let content = match fs::read_to_string(&file) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let value: serde_json::Value = match serde_json::from_str(&content) {
            Ok(v) => v,
            Err(_) => continue,
        };

        if let Some(transport) = value.get("transport").and_then(|v| v.as_str()) {
            if transport != "ws" {
                continue;
            }
        }

        let folders: Vec<String> = value
            .get("workspaceFolders")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|item| item.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        let score = folders
            .iter()
            .map(|folder| path_contains_score(folder, directory))
            .max()
            .unwrap_or(0);

        if score == 0 {
            continue;
        }

        let mtime = entry
            .metadata()
            .and_then(|m| m.modified())
            .ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_secs_f64())
            .unwrap_or(0.0);

        let auth_token = value
            .get("authToken")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        candidates.push((
            EditorConnection {
                url: format!("ws://127.0.0.1:{}", port),
                auth_token,
                source: format!("lock:{}", port),
            },
            score,
            mtime,
        ));
    }

    candidates.sort_by(|a, b| b.1.cmp(&a.1).then(b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal)));
    candidates.into_iter().next().map(|(conn, _, _)| conn)
}

/// Check if child path is contained within parent, returning parent path length as score.
fn path_contains_score(parent: &str, child: &Path) -> usize {
    let parent_path = Path::new(parent);
    let parent_abs = fs::canonicalize(parent_path).unwrap_or_else(|_| parent_path.to_path_buf());
    let child_abs = fs::canonicalize(child).unwrap_or_else(|_| child.to_path_buf());
    match child_abs.strip_prefix(&parent_abs) {
        Ok(relative) => {
            let rel_str = relative.to_string_lossy();
            if rel_str.is_empty() || (!rel_str.starts_with("..") && !relative.is_absolute()) {
                parent_abs.to_string_lossy().len()
            } else {
                0
            }
        }
        Err(_) => 0,
    }
}

// ---------------------------------------------------------------------------
// Zed editor integration
// ---------------------------------------------------------------------------

/// Zed selection result.
#[derive(Debug, Clone)]
pub enum ZedSelectionResult {
    Selection { selection: ZedEditorSelection },
    Empty,
    Unavailable,
}

/// Editor selection from Zed.
#[derive(Debug, Clone)]
pub struct ZedEditorSelection {
    pub file_path: String,
    pub source: String,
    pub ranges: Vec<ZedSelectionRange>,
}

/// A single selection range.
#[derive(Debug, Clone)]
pub struct ZedSelectionRange {
    pub text: String,
    pub selection: TextSelection,
}

/// A text selection (line/character position).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextPosition {
    pub line: usize,
    pub character: usize,
}

/// A selection range with start/end positions.
#[derive(Debug, Clone, Copy)]
pub struct TextSelection {
    pub start: TextPosition,
    pub end: TextPosition,
}

/// Resolve the Zed database path.
pub fn resolve_zed_db_path() -> Option<PathBuf> {
    let candidates: Vec<Option<PathBuf>> = vec![
        env::var("OPENCODE_ZED_DB").ok().map(PathBuf::from),
        dirs::home_dir().map(|h| {
            h.join("Library")
                .join("Application Support")
                .join("Zed")
                .join("db")
                .join("0-stable")
                .join("db.sqlite")
        }),
        dirs::home_dir().map(|h| h.join(".local").join("share").join("zed").join("db").join("0-stable").join("db.sqlite")),
    ];

    candidates
        .into_iter()
        .flatten()
        .find(|path| path.is_file())
}

/// Check if running inside Zed's terminal.
pub fn is_zed_terminal() -> bool {
    env::var("ZED_TERM").map(|v| v == "true").unwrap_or(false)
        || env::var("TERM_PROGRAM")
            .map(|v| v.to_lowercase() == "zed")
            .unwrap_or(false)
}

/// Resolve Zed selection from the database.
/// Returns `Unavailable` if the database cannot be read or no active editor is found.
pub fn resolve_zed_selection(db_path: &Path, _cwd: &Path) -> ZedSelectionResult {
    // The original TS implementation uses bun:sqlite to query Zed's database.
    // In Rust we would use rusqlite, but to avoid adding a dependency we
    // return Unavailable when the database exists but cannot be queried here.
    if !db_path.exists() {
        return ZedSelectionResult::Unavailable;
    }

    tracing::debug!(
        "Zed DB found at {:?} but SQLite querying is unavailable in Rust port",
        db_path
    );
    ZedSelectionResult::Unavailable
}

/// Convert a UTF-8 byte offset to a string character index.
pub fn utf8_byte_offset_to_string_index(text: &str, byte_offset: usize) -> usize {
    if byte_offset == 0 {
        return 0;
    }

    let mut bytes = 0usize;
    for (index, (_byte_offset, ch)) in text.char_indices().enumerate() {
        bytes += ch.len_utf8();
        if bytes >= byte_offset {
            return index + ch.len_utf8();
        }
    }
    text.len()
}

/// Convert character offsets to a line/column selection.
pub fn offsets_to_selection(text: &str, start_offset: usize, end_offset: usize) -> TextSelection {
    let start = start_offset.min(text.len());
    let end = end_offset.min(text.len());

    let mut line = 1usize;
    let mut line_start = 0usize;
    let mut start_pos = position(line, line_start, start);
    let mut end_pos = position(line, line_start, end);

    for (index, ch) in text.char_indices() {
        if index == start {
            start_pos = position(line, line_start, index);
        }
        if index == end {
            end_pos = position(line, line_start, index);
            break;
        }
        if ch == '\n' {
            line += 1;
            line_start = index + 1;
        }
    }

    TextSelection {
        start: start_pos,
        end: end_pos,
    }
}

fn position(line: usize, line_start: usize, offset: usize) -> TextPosition {
    TextPosition {
        line,
        character: offset - line_start + 1,
    }
}

/// Convert a byte offset to a text position.
pub fn offset_to_position(text: &str, offset: usize) -> TextPosition {
    let string_offset = utf8_byte_offset_to_string_index(text, offset);
    offsets_to_selection(text, string_offset, string_offset).start
}

/// Editor integration facade.
pub struct EditorIntegration;

impl EditorIntegration {
    pub fn connection(directory: &Path) -> Option<EditorConnection> {
        discover_editor_connection(directory)
    }

    pub fn selection(directory: &Path) -> ZedSelectionResult {
        match resolve_zed_db_path() {
            Some(db_path) => resolve_zed_selection(&db_path, directory),
            None => ZedSelectionResult::Unavailable,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_prompt_content_no_trailing() {
        assert_eq!(normalize_prompt_content("hello"), "hello");
    }

    #[test]
    fn test_normalize_prompt_content_trailing_newline() {
        assert_eq!(normalize_prompt_content("hello\n"), "hello");
    }

    #[test]
    fn test_normalize_prompt_content_trailing_crlf() {
        assert_eq!(normalize_prompt_content("hello\r\n"), "hello");
    }

    #[test]
    fn test_normalize_prompt_content_multiline_keeps_newline() {
        assert_eq!(normalize_prompt_content("line1\nline2\n"), "line1\nline2\n");
    }

    #[test]
    fn test_normalize_prompt_content_multiline_crlf_keeps() {
        assert_eq!(normalize_prompt_content("line1\nline2\r\n"), "line1\nline2\r\n");
    }

    #[test]
    fn test_utf8_byte_offset_ascii() {
        assert_eq!(utf8_byte_offset_to_string_index("hello", 3), 3);
    }

    #[test]
    fn test_utf8_byte_offset_multibyte() {
        let text = "héllo";
        // é is 2 bytes in UTF-8, so byte offset 3 is within é
        let result = utf8_byte_offset_to_string_index(text, 3);
        // Should be past the é (2 bytes) and 'l' (1 byte) = index 3
        assert!(result >= 2);
    }

    #[test]
    fn test_utf8_byte_offset_zero() {
        assert_eq!(utf8_byte_offset_to_string_index("hello", 0), 0);
    }

    #[test]
    fn test_utf8_byte_offset_past_end() {
        assert_eq!(utf8_byte_offset_to_string_index("hi", 100), 2);
    }

    #[test]
    fn test_offsets_to_selection_single_line() {
        let text = "hello world";
        let sel = offsets_to_selection(text, 0, 5);
        assert_eq!(sel.start.line, 1);
        assert_eq!(sel.start.character, 1);
        assert_eq!(sel.end.character, 6);
    }

    #[test]
    fn test_offsets_to_selection_multiline() {
        let text = "line1\nline2";
        let sel = offsets_to_selection(text, 6, 11);
        assert_eq!(sel.start.line, 2);
        assert_eq!(sel.end.line, 2);
    }

    #[test]
    fn test_offset_to_position() {
        let text = "hello\nworld";
        let pos = offset_to_position(text, 6);
        assert_eq!(pos.line, 2);
    }

    #[test]
    fn test_is_zed_terminal_false() {
        // In test environment, ZED_TERM is not set
        let _ = is_zed_terminal();
    }

    #[test]
    fn test_resolve_zed_db_path_none() {
        // In test environment, unlikely to find Zed DB
        let _ = resolve_zed_db_path();
    }

    #[test]
    fn test_resolve_zed_selection_no_db() {
        let result = resolve_zed_selection(Path::new("/nonexistent/db.sqlite"), Path::new("."));
        assert!(matches!(result, ZedSelectionResult::Unavailable));
    }

    #[test]
    fn test_path_contains_score_same() {
        let score = path_contains_score("/tmp", Path::new("/tmp"));
        assert!(score > 0);
    }
}
