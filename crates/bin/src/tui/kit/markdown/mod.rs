//! Streaming Markdown renderer.
//!
//! Parses accumulated assistant text into blocks (heading / paragraph /
//! list / code_block / table) and renders them as ratatui `Line`s. Each
//! render frame re-parses from scratch; unclosed fences/tables are shown
//! as in-progress so the user sees partial output while the model streams.
//!
//! Design follows claude-code-book Ch13 "streaming first": the renderer
//! never waits for "complete" markdown — it shows whatever has arrived.

pub mod scan;
pub mod span_style;
pub mod types;

pub use scan::scan_blocks;
pub use span_style::parse_inline;
pub use types::{line_from_spans, Block, SpanKind, StyledSpan};

use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;

use crate::tui::theme::Theme;

/// Render accumulated markdown `text` into a Paragraph widget styled by
/// `theme`. The Paragraph wraps at `width`.
pub fn render_paragraph<'a>(text: &'a str, width: usize, theme: &Theme) -> Paragraph<'a> {
    let lines = render_to_lines(text, width, theme);
    Paragraph::new(lines).wrap(ratatui::widgets::Wrap { trim: false })
}

/// Render accumulated markdown `text` into a `Vec<Line>` at `width`.
pub fn render_to_lines(text: &str, width: usize, theme: &Theme) -> Vec<Line<'static>> {
    if width == 0 {
        return vec![Line::from("")];
    }
    let blocks = scan_blocks(text);
    let mut out: Vec<Line<'static>> = Vec::new();

    for block in blocks {
        match block {
            Block::Blank => out.push(Line::from("")),
            Block::Heading { level, text } => {
                let (color, modi) = match level {
                    1 | 2 => (theme.markdown_heading, Modifier::BOLD),
                    _ => (theme.accent, Modifier::BOLD),
                };
                let style = Style::default().fg(color).add_modifier(modi);
                let prefix = "  ".repeat(level.saturating_sub(1) as usize);
                let mut current = String::from(&prefix);
                for word in text.split_whitespace() {
                    if current.chars().count() + word.chars().count() + 1 > width && !current.is_empty() {
                        out.push(Line::from(Span::styled(std::mem::take(&mut current), style)));
                    }
                    if !current.is_empty() {
                        current.push(' ');
                    }
                    current.push_str(word);
                }
                if !current.is_empty() {
                    out.push(Line::from(Span::styled(current, style)));
                }
            }
            Block::Paragraph { text } => {
                let spans = parse_inline(text);
                out.extend(line_from_spans(&spans, width, theme.text, theme.syntax_string, theme.markdown_link));
            }
            Block::ListItem { ordered, depth, text } => {
                let bullet = if ordered { format!("{}. ", depth + 1) } else { "• ".to_string() };
                let indent = "  ".repeat(depth as usize);
                let prefix = format!("{}{}", indent, bullet);
                let prefix_len = prefix.chars().count();
                let spans = parse_inline(text);
                let mut sub_lines = line_from_spans(&spans, width.saturating_sub(prefix_len), theme.text, theme.syntax_string, theme.markdown_link);
                for (i, line) in sub_lines.iter_mut().enumerate() {
                    if i == 0 {
                        let mut new_spans: Vec<Span<'static>> = vec![Span::raw(prefix.clone())];
                        new_spans.extend(line.spans.clone());
                        *line = Line::from(new_spans);
                    } else {
                        let pad = " ".repeat(prefix_len);
                        let mut new_spans: Vec<Span<'static>> = vec![Span::raw(pad)];
                        new_spans.extend(line.spans.clone());
                        *line = Line::from(new_spans);
                    }
                }
                out.extend(sub_lines);
            }
            Block::CodeBlock { lang, code, closed } => {
                let fg = theme.syntax_comment;
                let bg = theme.background_panel;
                // Header line: ```lang  [or "··· streaming" if not closed]
                let header = if closed {
                    format!("```{}", lang)
                } else {
                    format!("```{} (streaming…)", lang)
                };
                out.push(Line::from(Span::styled(header, Style::default().fg(fg).bg(bg))));
                for raw in code.lines() {
                    let mut padded = raw.to_string();
                    // Pad to width so bg fills the row (best-effort, may overflow).
                    let pad = width.saturating_sub(raw.chars().count() + 1);
                    padded.push_str(&" ".repeat(pad));
                    out.push(Line::from(Span::styled(padded, Style::default().fg(theme.text).bg(bg))));
                }
                if closed {
                    out.push(Line::from(Span::styled("```", Style::default().fg(fg).bg(bg))));
                }
            }
            Block::Table { header, rows, closed } => {
                // Simple render: each cell padded to column width.
                let cols = header.len();
                let mut widths = vec![0usize; cols];
                for (i, h) in header.iter().enumerate() {
                    widths[i] = widths[i].max(h.chars().count());
                }
                for row in &rows {
                    for (i, c) in row.iter().enumerate() {
                        if i < widths.len() {
                            widths[i] = widths[i].max(c.chars().count());
                        }
                    }
                }
                let render_row = |cells: &[&str]| -> Line<'static> {
                    let mut spans: Vec<Span<'static>> = vec![Span::raw("|")];
                    for (i, w) in widths.iter().enumerate() {
                        let c = cells.get(i).copied().unwrap_or("");
                        let pad = w.saturating_sub(c.chars().count());
                        spans.push(Span::raw(format!(" {}{} |", c, " ".repeat(pad))));
                    }
                    Line::from(spans)
                };
                out.push(render_row(&header));
                let sep: String = widths.iter().map(|w| format!(":{}|", "-".repeat(*w + 1))).collect();
                out.push(Line::from(Span::styled(sep, Style::default().fg(theme.border))));
                for row in &rows {
                    out.push(render_row(row));
                }
                if !closed {
                    out.push(Line::from(Span::styled("(table streaming…)", Style::default().fg(theme.text_muted))));
                }
            }
        }
    }

    if out.is_empty() {
        out.push(Line::from(""));
    }
    out
}
