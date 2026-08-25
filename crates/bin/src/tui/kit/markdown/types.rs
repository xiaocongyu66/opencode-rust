//! Block and span types for the streaming Markdown renderer.

use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};

/// A parsed block-level Markdown element. Streaming-friendly: an open code
/// fence produces `CodeBlock { closed: false }` so the renderer can show the
/// partial block while the model is still emitting it.
#[derive(Debug, Clone)]
pub enum Block<'a> {
    /// `#`, `##`, `###` ... heading. `level` = number of leading `#`.
    Heading { level: u8, text: &'a str },
    /// Plain paragraph text (may contain inline spans).
    Paragraph { text: &'a str },
    /// List item. `ordered` distinguishes `-`/`*` from `1.`. `depth` is
    /// the indent level (0 = top level).
    ListItem { ordered: bool, depth: u8, text: &'a str },
    /// Fenced code block. `lang` is the info string after ` ``` `. When
    /// `closed` is false the closing fence has not arrived yet.
    CodeBlock { lang: &'a str, code: &'a str, closed: bool },
    /// GFM table. `header` is the first row; `rows` are data rows. When
    /// `closed` is false the separator row hasn't arrived yet.
    Table { header: Vec<&'a str>, rows: Vec<Vec<&'a str>>, closed: bool },
    /// Blank line separator.
    Blank,
}

/// Inline span styles applied within a paragraph or list item.
#[derive(Debug, Clone, Copy)]
pub enum SpanKind {
    Plain,
    Code,
    Bold,
    Italic,
    Link,
}

/// Resolve a SpanKind to a ratatui Style given theme colors.
pub fn span_style(kind: SpanKind, text_color: Color, syntax_color: Color, link_color: Color) -> Style {
    match kind {
        SpanKind::Plain => Style::default().fg(text_color),
        SpanKind::Code => Style::default().fg(syntax_color),
        SpanKind::Bold => Style::default().fg(text_color).add_modifier(Modifier::BOLD),
        SpanKind::Italic => Style::default().fg(text_color).add_modifier(Modifier::ITALIC),
        SpanKind::Link => Style::default().fg(link_color).add_modifier(Modifier::UNDERLINED),
    }
}

/// A styled span produced by the inline parser.
#[derive(Debug, Clone)]
pub struct StyledSpan {
    pub text: String,
    pub kind: SpanKind,
}

impl StyledSpan {
    pub fn to_ratatui(&self, text_color: Color, syntax_color: Color, link_color: Color) -> Span<'static> {
        Span::styled(self.text.clone(), span_style(self.kind, text_color, syntax_color, link_color))
    }
}

/// Flatten a list of styled spans into a ratatui Line, wrapping at `width`.
pub fn line_from_spans(spans: &[StyledSpan], width: usize, text_color: Color, syntax_color: Color, link_color: Color) -> Vec<Line<'static>> {
    if width == 0 {
        return vec![Line::from("")];
    }
    // Tokenize into (text, style) atoms that can be re-wrapped.
    let mut atoms: Vec<(String, SpanKind)> = Vec::new();
    for s in spans {
        for (i, word) in s.text.split_whitespace().enumerate() {
            if i > 0 {
                atoms.push((" ".to_string(), SpanKind::Plain));
            }
            atoms.push((word.to_string(), s.kind));
        }
    }

    let mut out: Vec<Line<'static>> = Vec::new();
    let mut current: Vec<Span<'static>> = Vec::new();
    let mut current_len = 0usize;
    for (text, kind) in atoms {
        let w = text.chars().count();
        if current_len + w > width && !current.is_empty() {
            out.push(Line::from(std::mem::take(&mut current)));
            current_len = 0;
        }
        current.push(Span::styled(text, span_style(kind, text_color, syntax_color, link_color)));
        current_len += w;
    }
    if !current.is_empty() {
        out.push(Line::from(current));
    }
    if out.is_empty() {
        out.push(Line::from(""));
    }
    out
}
