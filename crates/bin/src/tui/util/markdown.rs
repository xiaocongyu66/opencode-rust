//! Markdown rendering — parse markdown with `pulldown-cmark` and render
//! to ratatui `Line<'static>` sequences styled by the current theme.
//!
//! Supported elements:
//! - Headings (H1–H6) — bold + heading color, with underline for H1/H2
//! - Paragraphs — wrapped to width, with inline styling
//! - Code blocks — fenced, with language tag + left border
//! - Inline code — code color
//! - Block quotes — left border + block_quote color
//! - Lists (ordered/unordered) — list_item color + bullet/number prefix
//! - Links — link_text color with URL in link color
//! - Strong/emph — bold/italic modifiers
//! - Horizontal rule — a line of `─` chars
//! - Soft/Hard breaks — newline within paragraph

use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::tui::theme::Theme;

/// Render markdown text into a sequence of ratatui lines, styled by `theme`.
/// `width` controls word-wrapping for paragraphs.
pub fn render_markdown_to_lines(text: &str, theme: &Theme, width: u16) -> Vec<Line<'static>> {
    let mut renderer = MarkdownRenderer::new(theme, width);
    renderer.render(text);
    renderer.finish()
}

/// Render markdown text directly into a frame area (widget-based).
pub fn render_markdown(f: &mut Frame, area: ratatui::layout::Rect, text: &str, theme: &Theme, width: u16) {
    let lines = render_markdown_to_lines(text, theme, width);
    f.render_widget(Paragraph::new(lines), area);
}

struct MarkdownRenderer<'a> {
    theme: &'a Theme,
    width: u16,
    lines: Vec<Line<'static>>,
    /// Current paragraph buffer: a sequence of spans that will be wrapped
    /// into lines on flush.
    pending_spans: Vec<Span<'static>>,
    /// Current text (plain) of pending_spans, for word-wrap calculations.
    pending_text: String,
    /// Stack of active inline styles (strong, emph, code, link).
    style_stack: Vec<InlineStyle>,
    /// Inside a code block? When non-empty, we are inside a fenced code
    /// block; the String is the language tag (possibly empty).
    code_block_lang: Option<String>,
    /// Buffer for code block contents.
    code_block_lines: Vec<String>,
    /// Inside a block quote?
    quote_depth: usize,
    /// List nesting depth (for indentation).
    list_stack: Vec<ListLevel>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum InlineStyle {
    Strong,
    Emph,
    Code,
    Link,
}

#[derive(Debug, Clone)]
struct ListLevel {
    /// 0-based item index within this list level.
    index: usize,
    /// Is this an ordered list?
    ordered: bool,
    /// Start number for ordered lists.
    start: usize,
}

impl<'a> MarkdownRenderer<'a> {
    fn new(theme: &'a Theme, width: u16) -> Self {
        Self {
            theme,
            width,
            lines: Vec::new(),
            pending_spans: Vec::new(),
            pending_text: String::new(),
            style_stack: Vec::new(),
            code_block_lang: None,
            code_block_lines: Vec::new(),
            quote_depth: 0,
            list_stack: Vec::new(),
        }
    }

    fn render(&mut self, text: &str) {
        let opts = Options::ENABLE_TABLES
            | Options::ENABLE_STRIKETHROUGH
            | Options::ENABLE_TASKLISTS;
        let parser = Parser::new_ext(text, opts);
        for event in parser {
            self.handle_event(event);
        }
        self.flush_paragraph();
        self.flush_code_block();
    }

    fn finish(self) -> Vec<Line<'static>> {
        self.lines
    }

    fn handle_event(&mut self, event: Event) {
        match event {
            Event::Start(tag) => self.start_tag(tag),
            Event::End(end) => self.end_tag(end),
            Event::Text(s) => self.text(&s),
            Event::Code(s) => self.inline_code(&s),
            Event::SoftBreak => self.soft_break(),
            Event::HardBreak => self.hard_break(),
            Event::Rule => self.horizontal_rule(),
            Event::FootnoteReference(_) => {}
            Event::TaskListMarker(checked) => self.task_list_marker(checked),
            Event::Html(s) => self.text(&s),
            Event::InlineHtml(s) => self.text(&s),
            Event::DisplayMath(_) | Event::InlineMath(_) => {}
        }
    }

    fn start_tag(&mut self, tag: Tag) {
        match tag {
            Tag::Paragraph => {
                // paragraphs flush any preceding block-level buffer first
                self.flush_code_block();
            }
            Tag::Heading { level, .. } => {
                self.flush_paragraph();
                self.flush_code_block();
                self.pending_spans.push(Span::styled(
                    String::new(),
                    Style::default()
                        .fg(self.theme.markdown_heading)
                        .add_modifier(Modifier::BOLD),
                ));
                let _ = level;
            }
            Tag::CodeBlock(kind) => {
                self.flush_paragraph();
                self.code_block_lang = Some(match kind {
                    pulldown_cmark::CodeBlockKind::Fenced(lang) => lang.into_string(),
                    pulldown_cmark::CodeBlockKind::Indented => String::new(),
                });
                self.code_block_lines.clear();
            }
            Tag::BlockQuote(_) => {
                self.flush_paragraph();
                self.quote_depth += 1;
            }
            Tag::List(start) => {
                self.flush_paragraph();
                let ordered = start.is_some();
                let start_val = start.unwrap_or(0) as usize;
                self.list_stack.push(ListLevel {
                    index: 0,
                    ordered,
                    start: start_val,
                });
            }
            Tag::Item => {
                self.flush_paragraph();
                // start a new list item paragraph
                if let Some(level) = self.list_stack.last_mut() {
                    level.index += 1;
                }
            }
            Tag::Emphasis => self.style_stack.push(InlineStyle::Emph),
            Tag::Strong => self.style_stack.push(InlineStyle::Strong),
            Tag::Strikethrough => self.style_stack.push(InlineStyle::Emph),
            Tag::Link { .. } => self.style_stack.push(InlineStyle::Link),
            Tag::Image { .. } => self.style_stack.push(InlineStyle::Link),
            Tag::FootnoteDefinition(_) => {}
            Tag::Table(_) | Tag::TableHead | Tag::TableRow | Tag::TableCell => {
                // tables fall back to plain text (one line per cell row)
            }
            Tag::DefinitionList | Tag::DefinitionListTitle | Tag::DefinitionListDefinition => {}
            // Unhandled block tags — ignore silently.
            _ => {}
        }
    }

    fn end_tag(&mut self, end: TagEnd) {
        match end {
            TagEnd::Paragraph => {
                self.flush_paragraph();
            }
            TagEnd::Heading(_) => {
                self.flush_paragraph();
                self.lines.push(Line::from(""));
            }
            TagEnd::CodeBlock => {
                self.flush_code_block();
            }
            TagEnd::BlockQuote(_) => {
                self.quote_depth = self.quote_depth.saturating_sub(1);
                // No empty line after block quote — keeps it compact.
            }
            TagEnd::List(_) => {
                self.list_stack.pop();
                // No empty line after list — keeps it compact.
            }
            TagEnd::Item => {
                self.flush_paragraph();
            }
            TagEnd::Emphasis | TagEnd::Strong | TagEnd::Strikethrough | TagEnd::Link | TagEnd::Image => {
                self.style_stack.pop();
            }
            _ => {}
        }
    }

    fn text(&mut self, s: &str) {
        if self.code_block_lang.is_some() {
            self.code_block_lines.push(s.to_string());
            return;
        }
        let style = self.current_style();
        // For headings, text should inherit heading style.
        self.pending_text.push_str(s);
        self.pending_spans
            .push(Span::styled(s.to_string(), style));
    }

    fn inline_code(&mut self, s: &str) {
        if self.code_block_lang.is_some() {
            // Shouldn't normally happen, but handle gracefully.
            self.code_block_lines.push(s.to_string());
            return;
        }
        let style = Style::default().fg(self.theme.markdown_code);
        self.pending_text.push_str(s);
        self.pending_spans.push(Span::styled(s.to_string(), style));
    }

    fn soft_break(&mut self) {
        if self.code_block_lang.is_some() {
            self.code_block_lines.push(String::new());
            return;
        }
        // soft break becomes a space within the paragraph
        self.pending_text.push(' ');
        self.pending_spans.push(Span::raw(" "));
    }

    fn hard_break(&mut self) {
        if self.code_block_lang.is_some() {
            self.code_block_lines.push(String::new());
            return;
        }
        self.flush_paragraph();
    }

    fn horizontal_rule(&mut self) {
        self.flush_paragraph();
        let w = self.width.max(1) as usize;
        let line: String = "─".chars().take(w).collect();
        self.lines.push(Line::from(Span::styled(
            line,
            Style::default().fg(self.theme.markdown_horizontal_rule),
        )));
        self.lines.push(Line::from(""));
    }

    fn task_list_marker(&mut self, checked: bool) {
        let glyph = if checked { "[x] " } else { "[ ] " };
        let style = Style::default().fg(self.theme.markdown_list_item);
        self.pending_text.push_str(glyph);
        self.pending_spans.push(Span::styled(glyph.to_string(), style));
    }

    fn current_style(&self) -> Style {
        let mut style = Style::default().fg(self.theme.markdown_text);
        for s in &self.style_stack {
            match s {
                InlineStyle::Strong => {
                    style = style.add_modifier(Modifier::BOLD);
                }
                InlineStyle::Emph => {
                    style = style.add_modifier(Modifier::ITALIC);
                }
                InlineStyle::Code => {
                    style = Style::default().fg(self.theme.markdown_code);
                }
                InlineStyle::Link => {
                    style = Style::default().fg(self.theme.markdown_link_text);
                }
            }
        }
        style
    }

    fn flush_paragraph(&mut self) {
        if self.pending_spans.is_empty() && self.pending_text.is_empty() {
            return;
        }

        // Build the list-item prefix if we're inside a list.
        let prefix = self.list_item_prefix();
        let prefix_len = prefix.chars().count();

        // Word-wrap the pending spans into lines of at most `width` chars.
        let max_w = self.width.max(1) as usize;
        // Account for the quote prefix (▎ + space per nesting level) so
        // quoted text doesn't overflow the area width.
        let quote_prefix_len: usize = self.quote_prefix().iter().map(|s| s.content.chars().count()).sum();
        let wrap_w = max_w.saturating_sub(self.list_indent_chars() + prefix_len + quote_prefix_len);
        let wrap_w = wrap_w.max(10);

        let quote_prefix = self.quote_prefix();
        // Thinking/reasoning block: use a subtle background tint to distinguish
        // it from regular text — matches Claude's thinking mode visual.
        let quote_bg = if self.quote_depth > 0 {
            Some(self.theme.background_element)
        } else {
            None
        };

        if !prefix.is_empty() {
            // First line gets the prefix.
            let mut first_spans: Vec<Span<'static>> = Vec::new();
            for qp in &quote_prefix {
                let mut s = qp.clone();
                if let Some(bg) = quote_bg { s.style = s.style.bg(bg); }
                first_spans.push(s);
            }
            let mut p = Span::styled(prefix, Style::default().fg(self.theme.markdown_list_item));
            if let Some(bg) = quote_bg { p.style = p.style.bg(bg); }
            first_spans.push(p);
            // Take the first wrapped line from pending_spans.
            let wrapped = wrap_spans(&self.pending_spans, &self.pending_text, wrap_w);
            if let Some(first) = wrapped.first() {
                for sp in &first.spans {
                    let mut s = sp.clone();
                    if let Some(bg) = quote_bg { s.style = s.style.bg(bg); }
                    first_spans.push(s);
                }
                self.lines.push(Line::from(first_spans));
            }
            // Remaining lines get indented.
            for line in wrapped.iter().skip(1) {
                let mut spans: Vec<Span<'static>> = Vec::new();
                for qp in &quote_prefix {
                    let mut s = qp.clone();
                    if let Some(bg) = quote_bg { s.style = s.style.bg(bg); }
                    spans.push(s);
                }
                let pad = Span::raw(" ".repeat(prefix_len));
                spans.push(pad);
                for sp in &line.spans {
                    let mut s = sp.clone();
                    if let Some(bg) = quote_bg { s.style = s.style.bg(bg); }
                    spans.push(s);
                }
                self.lines.push(Line::from(spans));
            }
        } else {
            let wrapped = wrap_spans(&self.pending_spans, &self.pending_text, wrap_w);
            for line in wrapped {
                let mut spans: Vec<Span<'static>> = Vec::new();
                for qp in &quote_prefix {
                    let mut s = qp.clone();
                    if let Some(bg) = quote_bg { s.style = s.style.bg(bg); }
                    spans.push(s);
                }
                for sp in &line.spans {
                    let mut s = sp.clone();
                    if let Some(bg) = quote_bg { s.style = s.style.bg(bg); }
                    spans.push(s);
                }
                self.lines.push(Line::from(spans));
            }
        }

        self.pending_spans.clear();
        self.pending_text.clear();
    }

    fn flush_code_block(&mut self) {
        let lang = match self.code_block_lang.take() {
            Some(l) => l,
            None => return,
        };
        if self.code_block_lines.is_empty() {
            return;
        }

        // Title bar: " lang " in text_muted, or " code " if no language.
        let title = if lang.is_empty() {
            "code".to_string()
        } else {
            lang.clone()
        };

        self.lines.push(Line::from(Span::styled(
            format!("┌─ {} ", title),
            Style::default().fg(self.theme.border),
        )));

        // Use syntect for syntax highlighting when possible.
        let code_text: String = self.code_block_lines.join("");
        let highlighted = crate::tui::util::syntax::highlight_code(&code_text, &title);
        for hl_line in highlighted {
            // Prefix each highlighted line with the border.
            let mut spans: Vec<Span<'static>> = vec![Span::styled("│ ", Style::default().fg(self.theme.border))];
            spans.extend(hl_line.spans);
            self.lines.push(Line::from(spans));
        }

        self.lines.push(Line::from(Span::styled(
            "└─",
            Style::default().fg(self.theme.border),
        )));
        self.code_block_lines.clear();
    }

    fn list_item_prefix(&self) -> String {
        let level = match self.list_stack.last() {
            Some(l) => l,
            None => return String::new(),
        };
        let bullet = if level.ordered {
            format!("{}. ", level.start.saturating_add(level.index.saturating_sub(1)))
        } else {
            "• ".to_string()
        };
        bullet
    }

    fn list_indent_chars(&self) -> usize {
        // 2 spaces per nesting level
        self.list_stack.len().saturating_sub(1) * 2
    }

    fn quote_prefix(&self) -> Vec<Span<'static>> {
        let mut spans = Vec::new();
        for _ in 0..self.quote_depth {
            spans.push(Span::styled(
                "▎ ",
                Style::default().fg(self.theme.markdown_block_quote),
            ));
        }
        spans
    }
}

/// A wrapped line: a sequence of spans fitting within `width`.
struct WrappedLine {
    spans: Vec<Span<'static>>,
}

/// Word-wrap a sequence of spans (whose concatenated text is `text`) into
/// lines of at most `width` characters.
///
/// The wrapping preserves per-character styling by splitting spans at word
/// boundaries: each output span corresponds to exactly one word (or a
/// single-space separator). This avoids the bug where cloning an entire
/// span's content for each word it contained caused text duplication.
fn wrap_spans(spans: &[Span<'static>], text: &str, width: usize) -> Vec<WrappedLine> {
    if width == 0 || spans.is_empty() {
        return vec![WrappedLine {
            spans: spans.to_vec(),
        }];
    }

    // Flatten into (char, style) pairs so we can split spans arbitrarily.
    type CharStyle = ratatui::style::Style;
    let mut chars: Vec<(char, CharStyle)> = Vec::with_capacity(text.chars().count());
    for span in spans {
        for c in span.content.chars() {
            chars.push((c, span.style));
        }
    }

    // Tokenize into words. A "word" is a maximal run of non-whitespace chars.
    // We keep the style for each char so multi-styled words (e.g. a word
    // with an inline-code suffix) render correctly.
    #[derive(Clone)]
    struct Word {
        chars: Vec<(char, CharStyle)>,
    }
    let mut words: Vec<Word> = Vec::new();
    let mut cur: Vec<(char, CharStyle)> = Vec::new();
    for (c, style) in chars {
        if c.is_whitespace() {
            if !cur.is_empty() {
                words.push(Word { chars: cur.clone() });
                cur.clear();
            }
            // drop the whitespace; we rejoin words with a single space
        } else {
            cur.push((c, style));
        }
    }
    if !cur.is_empty() {
        words.push(Word { chars: cur });
    }

    if words.is_empty() {
        return Vec::new();
    }

    // Group consecutive same-styled chars within a word into a single span,
    // to keep the span count reasonable.
    fn word_to_spans(word: &Word) -> Vec<Span<'static>> {
        let mut spans: Vec<Span<'static>> = Vec::new();
        let mut buf = String::new();
        let mut cur_style: Option<CharStyle> = None;
        for (c, style) in &word.chars {
            if cur_style.is_none() {
                cur_style = Some(*style);
                buf.push(*c);
            } else if cur_style == Some(*style) {
                buf.push(*c);
            } else {
                spans.push(Span::styled(std::mem::take(&mut buf), cur_style.unwrap()));
                cur_style = Some(*style);
                buf.push(*c);
            }
        }
        if !buf.is_empty() {
            spans.push(Span::styled(buf, cur_style.unwrap()));
        }
        spans
    }

    let mut lines: Vec<WrappedLine> = Vec::new();
    let mut line_spans: Vec<Span<'static>> = Vec::new();
    let mut line_len = 0usize;

    for word in &words {
        let word_len = word.chars.len();
        if line_len + word_len + 1 > width && !line_spans.is_empty() {
            lines.push(WrappedLine { spans: std::mem::take(&mut line_spans) });
            line_len = 0;
        }
        if !line_spans.is_empty() {
            line_spans.push(Span::raw(" "));
            line_len += 1;
        }
        line_spans.extend(word_to_spans(word));
        line_len += word_len;
    }
    if !line_spans.is_empty() {
        lines.push(WrappedLine { spans: line_spans });
    }

    lines
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::theme::themes::opencode;

    fn theme() -> std::sync::Arc<Theme> {
        std::sync::Arc::new(opencode())
    }

    #[test]
    fn empty_input_yields_no_lines() {
        let t = theme();
        let lines = render_markdown_to_lines("", &t, 80);
        assert!(lines.is_empty());
    }

    #[test]
    fn underscores_in_identifiers_not_treated_as_emphasis() {
        // Regression: `OPENAI_API_KEY` was being parsed as emphasis by
        // pulldown-cmark (the `_API_KEY_` substring matches `_..._`),
        // causing the text to appear duplicated in the TUI.
        let t = theme();
        let text = "未找到 API 密钥。请设置 OPENAI_API_KEY 或 ANTHROPIC_API_KEY。";
        let lines = render_markdown_to_lines(text, &t, 80);
        let mut all_text = String::new();
        for line in &lines {
            for span in &line.spans {
                all_text.push_str(&span.content);
            }
        }
        eprintln!("REPRO OUTPUT: {all_text}");
        assert_eq!(
            all_text.matches("OPENAI_API_KEY").count(),
            1,
            "OPENAI_API_KEY should appear exactly once, got: {all_text}"
        );
        assert_eq!(
            all_text.matches("ANTHROPIC_API_KEY").count(),
            1,
            "ANTHROPIC_API_KEY should appear exactly once, got: {all_text}"
        );
    }

    #[test]
    fn paragraph_wraps_to_width() {
        let t = theme();
        let long = "word ".repeat(50);
        let lines = render_markdown_to_lines(&long, &t, 20);
        assert!(lines.len() > 1);
    }

    #[test]
    fn heading_renders_on_its_own_line() {
        let t = theme();
        let lines = render_markdown_to_lines("# Hello World", &t, 80);
        // Heading + trailing blank line
        assert!(lines.len() >= 2);
        let line1 = line_to_string(&lines[0]);
        assert!(line1.contains("Hello"));
    }

    #[test]
    fn code_block_has_border_and_lang_tag() {
        let t = theme();
        let input = "```rust\nfn main() {}\n```\n";
        let lines = render_markdown_to_lines(input, &t, 80);
        let joined: Vec<String> = lines.iter().map(line_to_string).collect();
        let text = joined.join("\n");
        assert!(text.contains("rust"), "missing language tag: {text}");
        assert!(text.contains("│"), "missing left border: {text}");
        assert!(text.contains("fn main()"), "missing code body: {text}");
        assert!(text.contains("┌"), "missing top border: {text}");
        assert!(text.contains("└"), "missing bottom border: {text}");
    }

    #[test]
    fn code_block_without_language_uses_code_label() {
        let t = theme();
        let input = "```\nplain\n```\n";
        let lines = render_markdown_to_lines(input, &t, 80);
        let joined: Vec<String> = lines.iter().map(line_to_string).collect();
        let text = joined.join("\n");
        assert!(text.contains("code"), "missing fallback label: {text}");
    }

    #[test]
    fn block_quote_has_quote_prefix() {
        let t = theme();
        let input = "> quoted text";
        let lines = render_markdown_to_lines(input, &t, 80);
        let joined: Vec<String> = lines.iter().map(line_to_string).collect();
        let text = joined.join("\n");
        assert!(text.contains("▎"), "missing quote prefix: {text}");
    }

    #[test]
    fn unordered_list_has_bullets() {
        let t = theme();
        let input = "- one\n- two\n- three\n";
        let lines = render_markdown_to_lines(input, &t, 80);
        let joined: Vec<String> = lines.iter().map(line_to_string).collect();
        let text = joined.join("\n");
        assert!(text.contains("•"), "missing bullet: {text}");
    }

    #[test]
    fn ordered_list_has_numbers() {
        let t = theme();
        let input = "1. first\n2. second\n";
        let lines = render_markdown_to_lines(input, &t, 80);
        let joined: Vec<String> = lines.iter().map(line_to_string).collect();
        let text = joined.join("\n");
        assert!(text.contains("1."), "missing number 1: {text}");
        assert!(text.contains("2."), "missing number 2: {text}");
    }

    #[test]
    fn horizontal_rule_renders() {
        let t = theme();
        let lines = render_markdown_to_lines("---\n", &t, 40);
        let joined: Vec<String> = lines.iter().map(line_to_string).collect();
        let text = joined.join("\n");
        assert!(text.contains('─'), "missing hr char: {text}");
    }

    #[test]
    fn inline_code_kept_as_text() {
        let t = theme();
        let input = "use `cargo build` to compile";
        let lines = render_markdown_to_lines(input, &t, 80);
        let joined: Vec<String> = lines.iter().map(line_to_string).collect();
        let text = joined.join("\n");
        assert!(text.contains("cargo build"), "inline code lost: {text}");
    }

    #[test]
    fn mixed_paragraphs_and_code() {
        let t = theme();
        let input = "Intro text.\n\n```python\nprint(1)\n```\n\nMore text.";
        let lines = render_markdown_to_lines(input, &t, 80);
        let joined: Vec<String> = lines.iter().map(line_to_string).collect();
        let text = joined.join("\n");
        assert!(text.contains("Intro"));
        assert!(text.contains("python"));
        assert!(text.contains("print(1)"));
        assert!(text.contains("More text"));
    }

    fn line_to_string(line: &Line<'_>) -> String {
        let mut s = String::new();
        for span in &line.spans {
            s.push_str(&span.content);
        }
        s
    }
}
