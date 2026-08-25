//! Session route — main session view with messages, prompt, and footer.
//! Ported from opencode/packages/tui/src/routes/session/index.tsx (2725 lines)

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

use crate::tui::app::{App, MessageRole};
use crate::tui::theme::Theme;

const SIDEBAR_WIDTH: u16 = 42;
const MAX_PROMPT_WIDTH: u16 = 75;

pub fn render(f: &mut Frame, app: &mut App) {
    let area = f.area();
    let theme = app.theme.clone();

    f.render_widget(
        Block::default().style(Style::default().bg(theme.background)),
        area,
    );

    let has_permission = app.permission_state.is_some();
    let has_question = app.question_state.is_some();
    let overlay_active = has_permission || has_question;

    let mut constraints: Vec<Constraint> = Vec::new();
    constraints.push(Constraint::Length(1));
    constraints.push(Constraint::Min(3));
    if overlay_active {
        constraints.push(Constraint::Length(10));
    }
    constraints.push(Constraint::Length(3));
    constraints.push(Constraint::Length(1));

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(area);

    let mut idx = 0usize;
    let header_area = chunks[idx]; idx += 1;
    let messages_area = chunks[idx]; idx += 1;
    let overlay_area = if overlay_active { Some(chunks[idx]) } else { None };
    if overlay_active { idx += 1; }
    let prompt_area = chunks[idx]; idx += 1;
    let footer_area = chunks[idx];

    render_header(f, app, header_area, &theme);

    let (main_area, sidebar_area) = if app.show_sidebar {
        let sw = SIDEBAR_WIDTH.min(messages_area.width.saturating_sub(20));
        let main = Rect {
            x: messages_area.x,
            y: messages_area.y,
            width: messages_area.width.saturating_sub(sw),
            height: messages_area.height,
        };
        let side = Rect {
            x: messages_area.x + messages_area.width.saturating_sub(sw),
            y: messages_area.y,
            width: sw,
            height: messages_area.height,
        };
        (main, Some(side))
    } else {
        (messages_area, None)
    };

    if let Some(side) = sidebar_area {
        app.sidebar.render(f, side, &theme, &mut app.click_registry);
    }

    render_messages(f, app, main_area, &theme);

    if let Some(ref mut perm) = app.permission_state {
        perm.render(f, overlay_area.unwrap(), &theme);
    } else if let Some(ref mut qs) = app.question_state {
        qs.render(f, overlay_area.unwrap(), &theme);
    }

    if !overlay_active {
        let total_width = f.area().width;
        let prompt_max = MAX_PROMPT_WIDTH.min(total_width.saturating_sub(4));
        let centered_prompt = Rect {
            x: prompt_area.x + (total_width - prompt_max) / 2,
            y: prompt_area.y,
            width: prompt_max,
            height: prompt_area.height,
        };
        app.prompt.render(f, centered_prompt, &theme);
        app.prompt.render_autocomplete(f, centered_prompt, &theme);
    }

    app.footer.render(f, footer_area, &theme);
}

fn render_header(f: &mut Frame, app: &App, area: Rect, theme: &Theme) {
    let mut spans = vec![Span::styled(
        format!(" rsopencode - {} ", app.footer.agent),
        Style::default().fg(theme.primary).add_modifier(Modifier::BOLD),
    )];

    if app.is_thinking {
        let spinner_line = app.spinner.line(theme);
        spans.push(Span::raw("  "));
        for s in spinner_line.spans {
            spans.push(s);
        }
    }

    let header = Block::default()
        .borders(Borders::BOTTOM)
        .border_style(Style::default().fg(theme.border))
        .style(Style::default().bg(theme.background));

    f.render_widget(header, area);
    f.render_widget(
        Paragraph::new(Line::from(spans)).style(Style::default().bg(theme.background)),
        area,
    );
}

fn render_messages(f: &mut Frame, app: &App, area: Rect, theme: &Theme) {
    let mut lines: Vec<Line> = Vec::new();
    let content_width = (area.width.saturating_sub(4)) as usize;

    for msg in &app.messages {
        match msg.role {
            MessageRole::User => render_user_message(&msg.text, theme, content_width, &mut lines),
            MessageRole::Assistant => {
                render_assistant_message(&msg.text, theme, content_width, &mut lines);
            }
            MessageRole::System => render_system_message(&msg.text, theme, content_width, &mut lines),
        }
        lines.push(Line::from(""));
    }

    let total = lines.len();
    let visible = area.height as usize;
    let scroll = app.messages_scroll.scroll.min(total.saturating_sub(visible));

    let para = Paragraph::new(lines)
        .scroll((scroll as u16, 0))
        .wrap(Wrap { trim: false });

    let block = Block::default()
        .borders(Borders::LEFT)
        .border_style(Style::default().fg(theme.border_subtle))
        .style(Style::default().bg(theme.background));

    let inner = Rect {
        x: area.x + 1,
        y: area.y,
        width: area.width.saturating_sub(1),
        height: area.height,
    };

    f.render_widget(block, area);
    f.render_widget(para, inner);
}

fn render_user_message(text: &str, theme: &Theme, width: usize, lines: &mut Vec<Line>) {
    lines.push(Line::from(vec![
        Span::styled("|", Style::default().fg(theme.secondary)),
        Span::styled(format!(" {}", crate::t!("tui.message.user_prefix")), Style::default().fg(theme.secondary).add_modifier(Modifier::BOLD)),
    ]));
    wrap_text(text, theme.text, width, lines);
}

fn render_assistant_message(text: &str, theme: &Theme, width: usize, lines: &mut Vec<Line>) {
    lines.push(Line::from(vec![
        Span::styled("|", Style::default().fg(theme.primary)),
        Span::styled(format!(" {}", crate::t!("tui.message.assistant_prefix")), Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
    ]));

    for raw_line in text.lines() {
        if raw_line.is_empty() {
            lines.push(Line::from(""));
            continue;
        }
        if let Some(h) = raw_line.strip_prefix("# ") {
            wrap_styled(h, theme.markdown_heading, Modifier::BOLD, width, lines);
            continue;
        }
        if let Some(h) = raw_line.strip_prefix("## ") {
            wrap_styled(h, theme.markdown_heading, Modifier::BOLD, width, lines);
            continue;
        }
        if let Some(h) = raw_line.strip_prefix("### ") {
            wrap_styled(h, theme.accent, Modifier::BOLD, width, lines);
            continue;
        }
        if raw_line.starts_with("- ") || raw_line.starts_with("* ") {
            let item = &raw_line[2..];
            let wrapped = wrap_spans(parse_inline(item, theme), width);
            for (i, span_line) in wrapped.into_iter().enumerate() {
                let mut prefixed = vec![Span::raw(if i == 0 { "  - " } else { "    " })];
                prefixed.extend(span_line);
                lines.push(Line::from(prefixed));
            }
            continue;
        }
        if let Some(rest) = numbered_prefix(raw_line) {
            let wrapped = wrap_spans(parse_inline(rest, theme), width);
            for (i, span_line) in wrapped.into_iter().enumerate() {
                let mut prefixed = vec![Span::raw(if i == 0 { "  " } else { "    " })];
                prefixed.extend(span_line);
                lines.push(Line::from(prefixed));
            }
            continue;
        }
        if raw_line.starts_with("```") {
            lines.push(Line::from(Span::styled(
                raw_line.to_string(),
                Style::default().fg(theme.syntax_comment),
            )));
            continue;
        }
        let wrapped = wrap_spans(parse_inline(raw_line, theme), width);
        lines.extend(wrapped);
    }
}

fn render_system_message(text: &str, theme: &Theme, width: usize, lines: &mut Vec<Line>) {
    lines.push(Line::from(vec![
        Span::styled("|", Style::default().fg(theme.text_muted)),
        Span::styled(format!(" {}", crate::t!("tui.message.system_prefix")), Style::default().fg(theme.text_muted).add_modifier(Modifier::BOLD)),
    ]));
    wrap_text(text, theme.text_muted, width, lines);
}

fn wrap_text(text: &str, color: ratatui::style::Color, width: usize, lines: &mut Vec<Line>) {
    let mut current = String::new();
    for word in text.split_whitespace() {
        if current.len() + word.len() + 1 > width && !current.is_empty() {
            lines.push(Line::from(Span::styled(current.clone(), Style::default().fg(color))));
            current.clear();
        }
        if !current.is_empty() {
            current.push(' ');
        }
        current.push_str(word);
    }
    let was_empty = current.is_empty();
    if !was_empty {
        lines.push(Line::from(Span::styled(current, Style::default().fg(color))));
    }
    if was_empty {
        lines.push(Line::from(""));
    }
}

fn wrap_styled(
    text: &str,
    color: ratatui::style::Color,
    modifier: Modifier,
    width: usize,
    lines: &mut Vec<Line>,
) {
    let style = Style::default().fg(color).add_modifier(modifier);
    let mut current = String::new();
    for word in text.split_whitespace() {
        if current.len() + word.len() + 1 > width && !current.is_empty() {
            lines.push(Line::from(Span::styled(current.clone(), style)));
            current.clear();
        }
        if !current.is_empty() {
            current.push(' ');
        }
        current.push_str(word);
    }
    let was_empty = current.is_empty();
    if !was_empty {
        lines.push(Line::from(Span::styled(current, style)));
    }
}

fn parse_inline(text: &str, theme: &Theme) -> Vec<Span<'static>> {
    let mut spans: Vec<Span<'static>> = Vec::new();
    let mut buf = String::new();
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if chars[i] == '`' {
            if !buf.is_empty() {
                spans.push(Span::styled(buf.clone(), Style::default().fg(theme.text)));
                buf.clear();
            }
            let start = i + 1;
            let mut end = start;
            while end < chars.len() && chars[end] != '`' {
                end += 1;
            }
            if end < chars.len() {
                let code: String = chars[start..end].iter().collect();
                spans.push(Span::styled(code, Style::default().fg(theme.syntax_string)));
                i = end + 1;
                continue;
            }
        }
        if i + 1 < chars.len() && chars[i] == '*' && chars[i + 1] == '*' {
            if !buf.is_empty() {
                spans.push(Span::styled(buf.clone(), Style::default().fg(theme.text)));
                buf.clear();
            }
            let start = i + 2;
            let mut end = start + 1;
            while end + 1 < chars.len() && !(chars[end] == '*' && chars[end + 1] == '*') {
                end += 1;
            }
            if end + 1 < chars.len() {
                let bold_text: String = chars[start..end].iter().collect();
                spans.push(Span::styled(
                    bold_text,
                    Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
                ));
                i = end + 2;
                continue;
            }
        }
        if chars[i] == '[' {
            let close = chars[i..].iter().position(|&c| c == ']');
            if let Some(ci) = close {
                let after = i + ci + 1;
                if after < chars.len() && chars[after] == '(' {
                    let url_close = chars[after..].iter().position(|&c| c == ')');
                    if let Some(ui) = url_close {
                        let link_text: String = chars[i + 1..i + ci].iter().collect();
                        if !buf.is_empty() {
                            spans.push(Span::styled(buf.clone(), Style::default().fg(theme.text)));
                            buf.clear();
                        }
                        spans.push(Span::styled(
                            link_text,
                            Style::default().fg(theme.markdown_link).add_modifier(Modifier::UNDERLINED),
                        ));
                        i = after + ui + 1;
                        continue;
                    }
                }
            }
        }
        buf.push(chars[i]);
        i += 1;
    }
    if !buf.is_empty() {
        spans.push(Span::styled(buf, Style::default().fg(theme.text)));
    }
    spans
}

fn wrap_spans(spans: Vec<Span<'static>>, width: usize) -> Vec<Line<'static>> {
    let items: Vec<(String, Style)> = spans
        .iter()
        .map(|s| (s.content.to_string(), s.style))
        .collect();

    let mut result: Vec<Line<'static>> = Vec::new();
    let mut current_spans: Vec<Span<'static>> = Vec::new();
    let mut current_len = 0usize;

    for (text, style) in items {
        let words: Vec<&str> = text.split_whitespace().collect();
        if words.is_empty() {
            continue;
        }
        for word in words {
            let word_len = word.len();
            if current_len + word_len + 1 > width && !current_spans.is_empty() {
                result.push(Line::from(std::mem::take(&mut current_spans)));
                current_len = 0;
            }
            if current_len > 0 {
                current_spans.push(Span::raw(" "));
                current_len += 1;
            }
            current_spans.push(Span::styled(word.to_string(), style));
            current_len += word_len;
        }
    }
    if !current_spans.is_empty() {
        result.push(Line::from(current_spans));
    }
    if result.is_empty() {
        result.push(Line::from(""));
    }
    result
}

fn numbered_prefix(line: &str) -> Option<&str> {
    let chars: Vec<char> = line.chars().collect();
    if chars.is_empty() {
        return None;
    }
    if !chars[0].is_ascii_digit() {
        return None;
    }
    let mut i = 0;
    while i < chars.len() && chars[i].is_ascii_digit() {
        i += 1;
    }
    if i < chars.len() && (chars[i] == '.' || chars[i] == ')') {
        i += 1;
    }
    if i < chars.len() && chars[i] == ' ' {
        i += 1;
    }
    let byte_pos = chars[..i].iter().map(|c| c.len_utf8()).sum::<usize>();
    if byte_pos < line.len() {
        Some(&line[byte_pos..])
    } else {
        Some("")
    }
}
