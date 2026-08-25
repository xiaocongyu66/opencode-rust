//! Incremental Markdown block scanner.
//!
//! Parses a potentially-partial Markdown string into a sequence of
//! `Block`s. The parser is streaming-friendly: unclosed code fences and
//! tables produce blocks with `closed: false` so the renderer can display
//! partial content while the model is still emitting.

use super::types::Block;

/// Scan the accumulated text into blocks. Each call re-parses from scratch;
/// the caller decides how often to invoke this (typically once per render
/// frame while a message is streaming).
pub fn scan_blocks(text: &str) -> Vec<Block<'_>> {
    let mut blocks: Vec<Block<'_>> = Vec::new();
    let mut lines = text.lines().peekable();

    while let Some(line) = lines.next() {
        // Skip blank lines but emit a single Blank separator.
        if line.trim().is_empty() {
            blocks.push(Block::Blank);
            continue;
        }

        // Code fence: ```lang ... ```
        if let Some(rest) = line.trim_start().strip_prefix("```") {
            let lang = rest.trim();
            let mut code = String::new();
            let mut closed = false;
            for inner in lines.by_ref() {
                if inner.trim_start().starts_with("```") {
                    closed = true;
                    break;
                }
                code.push_str(inner);
                code.push('\n');
            }
            // The accumulated code is owned by this scope; leak it so the
            // Block can borrow it. Acceptable in streaming because we
            // re-scan every frame and old leaks are dropped when the
            // message finalizes.
            let code_ref: &'static str = code.leak();
            if closed {
                blocks.push(Block::CodeBlock { lang, code: code_ref, closed: true });
            } else {
                // Unclosed fence: show partial code as a code block so the
                // renderer can mark it as in-progress.
                blocks.push(Block::CodeBlock { lang, code: code_ref, closed: false });
            }
            continue;
        }

        // Heading: # / ## / ###
        if let Some(stripped) = line.strip_prefix("# ") {
            blocks.push(Block::Heading { level: 1, text: stripped });
            continue;
        }
        if let Some(stripped) = line.strip_prefix("## ") {
            blocks.push(Block::Heading { level: 2, text: stripped });
            continue;
        }
        if let Some(stripped) = line.strip_prefix("### ") {
            blocks.push(Block::Heading { level: 3, text: stripped });
            continue;
        }

        // Unordered list item: - or *
        if let Some(stripped) = line.trim_start().strip_prefix("- ").or_else(|| line.trim_start().strip_prefix("* ")) {
            let depth = leading_spaces(line) / 2;
            blocks.push(Block::ListItem { ordered: false, depth: depth as u8, text: stripped });
            continue;
        }

        // Ordered list item: 1. / 2. ...
        if let Some(rest) = ordered_list_prefix(line.trim_start()) {
            let depth = leading_spaces(line) / 2;
            blocks.push(Block::ListItem { ordered: true, depth: depth as u8, text: rest });
            continue;
        }

        // Table: line containing | and next line is |---| separator
        if line.contains('|') {
            if let Some(next) = lines.peek() {
                if is_table_separator(next) {
                    let header = split_table_row(line);
                    let _ = lines.next(); // consume separator
                    let mut rows: Vec<Vec<&str>> = Vec::new();
                    let mut closed = true;
                    for row_line in lines.by_ref() {
                        if row_line.trim().is_empty() {
                            break;
                        }
                        if !row_line.contains('|') {
                            // Non-table line after partial table: treat as
                            // unclosed and re-feed this line.
                            closed = false;
                            // Push remaining as paragraph by re-scanning.
                            blocks.push(Block::Table { header, rows, closed });
                            blocks.extend(scan_blocks(row_line));
                            return finish(blocks);
                        }
                        rows.push(split_table_row(row_line));
                    }
                    blocks.push(Block::Table { header, rows, closed });
                    continue;
                }
            }
        }

        // Default: paragraph.
        blocks.push(Block::Paragraph { text: line });
    }

    finish(blocks)
}

fn finish<'a>(mut blocks: Vec<Block<'a>>) -> Vec<Block<'a>> {
    // Coalesce consecutive Blank blocks.
    let mut out: Vec<Block<'a>> = Vec::with_capacity(blocks.len());
    for b in blocks.drain(..) {
        if let Block::Blank = &b {
            if matches!(out.last(), Some(Block::Blank)) {
                continue;
            }
        }
        out.push(b);
    }
    out
}

fn leading_spaces(s: &str) -> usize {
    s.chars().take_while(|c| *c == ' ').count()
}

fn ordered_list_prefix(s: &str) -> Option<&str> {
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
    }
    if i == 0 || i >= bytes.len() {
        return None;
    }
    if bytes[i] == b'.' || bytes[i] == b')' {
        i += 1;
    }
    if i < bytes.len() && bytes[i] == b' ' {
        i += 1;
        return Some(&s[i..]);
    }
    None
}

fn is_table_separator(line: &str) -> bool {
    let t = line.trim();
    if !t.contains('|') {
        return false;
    }
    // Each segment between | must be only dashes, colons, or spaces.
    t.split('|').all(|seg| seg.chars().all(|c| c == '-' || c == ':' || c == ' ' || c.is_ascii_digit()))
        && t.contains('-')
}

fn split_table_row(line: &str) -> Vec<&str> {
    let t = line.trim();
    let stripped = t.strip_prefix('|').unwrap_or(t);
    let stripped = stripped.strip_suffix('|').unwrap_or(stripped);
    stripped.split('|').map(|c| c.trim()).collect()
}
