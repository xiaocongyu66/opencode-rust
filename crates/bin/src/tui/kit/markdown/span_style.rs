//! Inline span parser: `code`, **bold**, *italic*, [link](url).

use super::types::{SpanKind, StyledSpan};

/// Parse a line of inline markdown into styled spans.
pub fn parse_inline(text: &str) -> Vec<StyledSpan> {
    let mut spans: Vec<StyledSpan> = Vec::new();
    let chars: Vec<char> = text.chars().collect();
    let mut buf = String::new();
    let mut i = 0;

    while i < chars.len() {
        // `code` — backtick-delimited inline code.
        if chars[i] == '`' {
            if !buf.is_empty() {
                spans.push(StyledSpan { text: std::mem::take(&mut buf), kind: SpanKind::Plain });
            }
            let start = i + 1;
            let mut end = start;
            while end < chars.len() && chars[end] != '`' {
                end += 1;
            }
            if end < chars.len() {
                let code: String = chars[start..end].iter().collect();
                spans.push(StyledSpan { text: code, kind: SpanKind::Code });
                i = end + 1;
                continue;
            }
        }
        // **bold** or *italic*
        if chars[i] == '*' {
            if i + 1 < chars.len() && chars[i + 1] == '*' {
                if let Some((text_span, next_i)) = match_marker(&chars, i, "**", SpanKind::Bold) {
                    if !buf.is_empty() {
                        spans.push(StyledSpan { text: std::mem::take(&mut buf), kind: SpanKind::Plain });
                    }
                    spans.push(StyledSpan { text: text_span, kind: SpanKind::Bold });
                    i = next_i;
                    continue;
                }
            } else {
                if let Some((text_span, next_i)) = match_marker(&chars, i, "*", SpanKind::Italic) {
                    if !buf.is_empty() {
                        spans.push(StyledSpan { text: std::mem::take(&mut buf), kind: SpanKind::Plain });
                    }
                    spans.push(StyledSpan { text: text_span, kind: SpanKind::Italic });
                    i = next_i;
                    continue;
                }
            }
        }
        // [link](url)
        if chars[i] == '[' {
            if let Some((link_text, url, next_i)) = match_link(&chars, i) {
                let _ = url; // url unused in render; link text is what's shown
                if !buf.is_empty() {
                    spans.push(StyledSpan { text: std::mem::take(&mut buf), kind: SpanKind::Plain });
                }
                spans.push(StyledSpan { text: link_text, kind: SpanKind::Link });
                i = next_i;
                continue;
            }
        }
        buf.push(chars[i]);
        i += 1;
    }
    if !buf.is_empty() {
        spans.push(StyledSpan { text: buf, kind: SpanKind::Plain });
    }
    if spans.is_empty() {
        spans.push(StyledSpan { text: String::new(), kind: SpanKind::Plain });
    }
    spans
}

/// Match `marker ... marker` and return the inner text plus the index after
/// the closing marker.
fn match_marker(chars: &[char], start: usize, marker: &str, _kind: SpanKind) -> Option<(String, usize)> {
    let m: Vec<char> = marker.chars().collect();
    let inner_start = start + m.len();
    let mut end = inner_start;
    while end + m.len() <= chars.len() {
        if chars[end..end + m.len()] == m[..] {
            let text: String = chars[inner_start..end].iter().collect();
            return Some((text, end + m.len()));
        }
        end += 1;
    }
    None
}

/// Match `[text](url)` starting at `start`.
fn match_link(chars: &[char], start: usize) -> Option<(String, String, usize)> {
    // Find closing ]
    let close = chars[start + 1..].iter().position(|&c| c == ']')?;
    let text_end = start + 1 + close;
    let text: String = chars[start + 1..text_end].iter().collect();
    let after = text_end + 1;
    if after >= chars.len() || chars[after] != '(' {
        return None;
    }
    let url_close = chars[after + 1..].iter().position(|&c| c == ')')?;
    let url_end = after + 1 + url_close;
    let url: String = chars[after + 1..url_end].iter().collect();
    Some((text, url, url_end + 1))
}
