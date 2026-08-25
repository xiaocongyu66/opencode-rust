//! TUI rendering — 还原原版 routes/session/index.tsx + routes/home.tsx

use ratatui::layout::{Constraint, Direction, Layout, Rect, Alignment};
use ratatui::style::{Style, Modifier};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Clear};
use ratatui::Frame;

use crate::tui::app::{App, MessageRole};
use crate::tui::component::logo::render_logo;
use crate::tui::event::InputMode;
use crate::tui::theme::Theme;

pub fn render(f: &mut Frame, app: &mut App) {
    let area = f.area();
    let theme = app.theme.clone();
    f.render_widget(Block::default().style(Style::default().bg(theme.background)), area);

    if app.messages.len() <= 1 {
        render_home(f, app);
    } else {
        render_session(f, app);
    }

    if app.mode == InputMode::Help { render_help(f, area, app); }
    app.toast_manager.render(f, area, &theme);
    app.command_palette.render(f, area, &theme);
    if let Some(ref dialog) = app.dialog { dialog.render(f, area, &theme); }
}

fn render_home(f: &mut Frame, app: &mut App) {
    let theme = app.theme.clone();
    let area = f.area();
    let total_width = area.width;

    // Compact vertical layout: top spacer + logo + 1 gap + prompt + 1 gap + hint + bottom spacer + footer.
    // Matches the TS home.tsx: logo + 1 line + prompt + bottom area.
    let chunks = Layout::default().direction(Direction::Vertical).constraints([
        Constraint::Min(1),       // 0: top spacer (flex)
        Constraint::Length(4),     // 1: logo (4 lines)
        Constraint::Length(1),     // 2: gap
        Constraint::Length(7),     // 3: prompt (needs room for border + text + meta + hint)
        Constraint::Length(1),     // 4: hint
        Constraint::Min(1),        // 5: bottom spacer (flex)
        Constraint::Length(1),     // 6: footer
    ]).split(area);

    let logo_width = 44u16;
    let logo_area = Rect { x: chunks[1].x + (total_width.saturating_sub(logo_width)) / 2, y: chunks[1].y, width: logo_width, height: chunks[1].height };
    render_logo(logo_area, f.buffer_mut(), &theme);

    // Home prompt: centered, max width = max(75, 70% of terminal width).
    // Matches the TS home.tsx `promptMaxWidth` formula.
    let prompt_max = std::cmp::max(75, total_width * 7 / 10);
    let pw = std::cmp::min(prompt_max, total_width.saturating_sub(4));
    let pa = Rect { x: chunks[3].x + (total_width - pw) / 2, y: chunks[3].y, width: pw, height: chunks[3].height };
    app.prompt.render(f, pa, &theme);
    app.prompt.render_autocomplete(f, pa, &theme);
    // Register prompt meta line as clickable (model selection).
    if pa.height > 0 {
        let meta_area = Rect {
            x: pa.x + 4,
            y: pa.y + pa.height.saturating_sub(1),
            width: pa.width.saturating_sub(6),
            height: 1,
        };
        app.click_registry.register(
            meta_area,
            "model:select",
            Some("Click to select model".to_string()),
        );
    }

    let hint = if app.mode == InputMode::Insert { crate::t!("tui.hint.send").to_string() } else { crate::t!("tui.hint.quit").to_string() };
    let ha = Rect { x: chunks[4].x + (total_width.saturating_sub(hint.len() as u16)) / 2, y: chunks[4].y, width: hint.len().min(total_width as usize) as u16, height: 1 };
    f.render_widget(Paragraph::new(hint).style(Style::default().fg(theme.text_muted)).alignment(Alignment::Center), ha);

    app.footer.render(f, chunks[6], &theme);
}

fn render_session(f: &mut Frame, app: &mut App) {
    let theme = app.theme.clone();
    let area = f.area();
    let total_width = area.width;

    // Sidebar shows on the right of the session view. Visible when toggled
    // on (default: on for session view) — user can toggle with `s`.
    let sidebar_visible = app.show_sidebar;
    let sidebar_width: u16 = if sidebar_visible { 42 } else { 0 };
    let h_chunks = Layout::default().direction(Direction::Horizontal).constraints([
        Constraint::Min(1), Constraint::Length(sidebar_width),
    ]).split(area);

    let main_area = h_chunks[0];

    // 垂直布局：头部 + 消息 + 间距 + 输入框 + 底部
    // 加 1 行间距让消息内容和输入框分开,避免贴在一起。
    let v_chunks = Layout::default().direction(Direction::Vertical).constraints([
        Constraint::Length(1), Constraint::Min(1), Constraint::Length(1), Constraint::Length(6), Constraint::Length(1),
    ]).split(main_area);

    // 头部栏
    let header = Block::default().borders(Borders::BOTTOM).border_style(Style::default().fg(theme.border))
        .title(Span::styled(format!(" {} ", app.footer.agent), Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)))
        .style(Style::default().bg(theme.background));
    f.render_widget(header, v_chunks[0]);

    // 消息列表
    let msg_area = Rect { x: v_chunks[1].x + 1, y: v_chunks[1].y, width: v_chunks[1].width.saturating_sub(2), height: v_chunks[1].height };
    let mut lines: Vec<Line> = Vec::new();
    let mut message_offsets: Vec<usize> = Vec::new();
    lines.push(Line::from(""));

    for msg in &app.messages {
        // Record the starting line of this message for message-boundary jumps.
        message_offsets.push(lines.len());
        match msg.role {
            MessageRole::User => {
                // User message: left border (┃) + backgroundPanel + padding-left 2
                // Matches the TS `UserMessage` component (border-left + panel bg).
                let max_w = (msg_area.width.saturating_sub(4)) as usize;
                let panel_bg = theme.background_panel;
                let border_fg = theme.secondary;
                // Total width of the panel row (border + padding + content + right pad).
                // Pad each line to msg_area.width so the panel bg fills the full row
                // and doesn't leave a gap where the page background bleeds through.
                let row_width = msg_area.width as usize;
                // header line: border + "You" + panel fill to end
                {
                    let prefix = format!("┃ {}  ", crate::t!("tui.message.user_prefix"));
                    let prefix_len = prefix.chars().count();
                    let pad = row_width.saturating_sub(prefix_len);
                    lines.push(Line::from(vec![
                        Span::styled("┃ ", Style::default().fg(border_fg)),
                        Span::styled(crate::t!("tui.message.user_prefix").to_string(), Style::default().fg(border_fg).add_modifier(Modifier::BOLD)),
                        Span::styled(" ".repeat(pad), Style::default().bg(panel_bg)),
                    ]));
                }
                // body: each wrapped line prefixed with "┃ " border + padding
                let wrapped = wrap_words(&msg.text, max_w);
                for line in wrapped {
                    let content_len = 2 + 1 + line.chars().count() + 1; // "┃ " + " " + line + " "
                    let pad = row_width.saturating_sub(content_len);
                    lines.push(Line::from(vec![
                        Span::styled("┃ ", Style::default().fg(border_fg)),
                        Span::styled(" ", Style::default().bg(panel_bg)),
                        Span::styled(line, Style::default().fg(theme.text).bg(panel_bg)),
                        Span::styled(" ", Style::default().bg(panel_bg)),
                        Span::styled(" ".repeat(pad), Style::default().bg(panel_bg)),
                    ]));
                }
                // bottom padding line
                {
                    let prefix_len = 4; // "┃ " + "  "
                    let pad = row_width.saturating_sub(prefix_len);
                    // If this message is queued, show a QUEUED tag.
                    if msg.queued {
                        lines.push(Line::from(vec![
                            Span::styled("┃ ", Style::default().fg(border_fg)),
                            Span::styled("  ", Style::default().bg(panel_bg)),
                            Span::styled(" QUEUED ", Style::default().fg(theme.background).bg(border_fg).add_modifier(Modifier::BOLD)),
                            Span::styled(" ".repeat(pad.saturating_sub(8)), Style::default().bg(panel_bg)),
                        ]));
                    } else {
                        lines.push(Line::from(vec![
                            Span::styled("┃ ", Style::default().fg(border_fg)),
                            Span::styled("  ", Style::default().bg(panel_bg)),
                            Span::styled(" ".repeat(pad), Style::default().bg(panel_bg)),
                        ]));
                    }
                }
                lines.push(Line::from(""));
            }
            MessageRole::Assistant => {
                // Assistant message: ● prefix + indented content (no per-line
                // background tint). Visual contrast with user messages comes
                // from: (1) user has ┃ border + panel bg, assistant is plain;
                // (2) assistant content is indented by 3 spaces; (3) the
                // footer line at the bottom carries agent · model info.
                lines.push(Line::from(vec![
                    Span::styled("● ", Style::default().fg(theme.primary)),
                    Span::styled(crate::t!("tui.message.assistant_prefix").to_string(), Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
                ]));
                let _ = (msg_area.width.saturating_sub(6)) as usize; // max_w unused now

                // If the message has structured parts, render each one in turn.
                // Otherwise fall back to plain-text wrapping for legacy messages.
                if !msg.parts.is_empty() {
                    for part in &msg.parts {
                        match part {
                            crate::tui::app::ChatPart::Text { text } => {
                                // Assistant text is markdown — render with 3-space indent
                                // and a right margin so long lines don't bleed into the sidebar.
                                let md_lines = crate::tui::util::markdown::render_markdown_to_lines(
                                    text, &theme, msg_area.width.saturating_sub(6),
                                );
                                for ml in md_lines {
                                    let mut spans: Vec<Span> = Vec::with_capacity(ml.spans.len() + 1);
                                    spans.push(Span::raw("   "));
                                    spans.extend(ml.spans);
                                    lines.push(Line::from(spans));
                                }
                            }
                            crate::tui::app::ChatPart::Tool { .. } => {
                                crate::tui::component::tool_render::render_tool_part_to_lines(
                                    &mut lines, part, &theme, msg_area.width,
                                );
                            }
                        }
                    }
                } else {
                    // Legacy: no parts — treat as markdown.
                    let md_lines = crate::tui::util::markdown::render_markdown_to_lines(
                        &msg.text, &theme, msg_area.width.saturating_sub(6),
                    );
                    for ml in md_lines {
                        let mut spans: Vec<Span> = Vec::with_capacity(ml.spans.len() + 1);
                        spans.push(Span::raw("   "));
                        spans.extend(ml.spans);
                        lines.push(Line::from(spans));
                    }
                }

                // Footer: ▣ agent · model · duration
                // Matches TS: "▣ Build · model-name · 1.2s"
                let model_display = if app.current_model.is_empty() {
                    "model".to_string()
                } else {
                    app.current_model.clone()
                };
                // Separator line above the footer for clear visual boundary.
                lines.push(Line::from(Span::styled(
                    "   ──────────────────────────────────",
                    Style::default().fg(theme.border),
                )));
                lines.push(Line::from(vec![
                    Span::styled("   ▣ ", Style::default().fg(theme.primary)),
                    Span::styled(app.footer.agent.clone(), Style::default().fg(theme.text)),
                    Span::styled(" · ", Style::default().fg(theme.text_muted)),
                    Span::styled(model_display, Style::default().fg(theme.text_muted)),
                ]));
            }
            MessageRole::System => {
                // System message: muted ● prefix + wrapped text.
                // Error messages (containing [错误: or [Error:) get red highlight.
                let is_error = msg.text.starts_with("[错误") || msg.text.starts_with("[Error")
                    || msg.text.contains("LLM error") || msg.text.contains("504")
                    || msg.text.contains("500") || msg.text.contains("timeout");
                let prefix_color = if is_error { theme.error } else { theme.text_muted };
                let text_color = if is_error { theme.error } else { theme.text_muted };
                lines.push(Line::from(vec![
                    Span::styled("● ", Style::default().fg(prefix_color)),
                    Span::styled(crate::t!("tui.message.system_prefix").to_string(), Style::default().fg(prefix_color).add_modifier(Modifier::BOLD)),
                ]));
                let max_w = (msg_area.width.saturating_sub(2)) as usize;
                let wrapped = wrap_words(&msg.text, max_w);
                for line in wrapped {
                    lines.push(Line::from(Span::styled(line, Style::default().fg(text_color))));
                }
                lines.push(Line::from(""));
            }
        }
    }

    // While the assistant is thinking, show a spinner line at the end.
    // Uses the spinner's current mode (thinking/tool-use/responding) to pick
    // a color and verb label — mirrors claude-code-best's Spinner component.
    //
    // Only append the spinner row when there is no streaming assistant text
    // yet. Once text is arriving (current_assistant_text non-empty), the
    // assistant message in `app.messages` already holds the partial text and
    // auto-follow will pin to the bottom of it. Adding a spinner row below
    // that would push the real content up and make it look like the view is
    // stuck — the "can't scroll to see the latest" bug.
    if (app.is_thinking || app.spinner_active) && app.current_assistant_text.is_empty() {
        lines.push(Line::from(""));
        let spinner_fg = app.spinner.mode.color(&theme);
        lines.push(Line::from(vec![
            Span::styled(app.spinner.current_frame().to_string(), Style::default().fg(spinner_fg)),
            Span::raw(" "),
            Span::styled(app.spinner.display_label(), Style::default().fg(spinner_fg)),
        ]));
    }

    // Hand off to the ScrollView — it renders the paragraph, draws the
    // optional scrollbar, and registers the track as a draggable region.
    app.messages_scroll.message_offsets = message_offsets;
    app.messages_scroll.render(f, msg_area, lines, &mut app.click_registry);

    // 输入框
    let pw = total_width.saturating_sub(4);
    // Prompt: full width of the main area (left of sidebar), with side margins.
    let main_w = main_area.width;
    let pw = main_w.saturating_sub(4);
    let pa = Rect { x: v_chunks[3].x + 2, y: v_chunks[3].y, width: pw, height: v_chunks[3].height };
    app.prompt.render(f, pa, &theme);
    app.prompt.render_autocomplete(f, pa, &theme);

    // Register the prompt meta line (agent · model · provider) as clickable —
    // clicking it opens the model selection dialog.
    if pa.height > 0 {
        let meta_area = Rect {
            x: pa.x + 4,
            y: pa.y + pa.height.saturating_sub(1),
            width: pa.width.saturating_sub(6),
            height: 1,
        };
        app.click_registry.register(
            meta_area,
            "model:select",
            Some("Click to select model".to_string()),
        );
    }

    // 底部状态栏
    app.footer.render(f, v_chunks[4], &theme);

    // 侧边栏
    if app.show_sidebar {
        app.sidebar.render(f, h_chunks[1], &theme, &mut app.click_registry);
    }
}

fn render_help(f: &mut Frame, area: Rect, app: &App) {
    let theme = app.theme.clone();
    let lines = vec![
        Line::from(Span::styled(crate::t!("tui.help.title").to_string(), Style::default().fg(theme.primary).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(vec![Span::styled("  Enter", Style::default().fg(theme.accent)), Span::raw("         "), Span::raw(crate::t!("tui.prompt.send").to_string())]),
        Line::from(vec![Span::styled("  Shift+Enter", Style::default().fg(theme.accent)), Span::raw("   "), Span::raw(crate::t!("tui.prompt.newline").to_string())]),
        Line::from(vec![Span::styled("  Esc", Style::default().fg(theme.accent)), Span::raw("          "), Span::raw(crate::t!("tui.prompt.esc").to_string())]),
        Line::from(vec![Span::styled("  /", Style::default().fg(theme.accent)), Span::raw("            "), Span::raw(crate::t!("tui.hint.slash").to_string())]),
        Line::from(vec![Span::styled("  :", Style::default().fg(theme.accent)), Span::raw("            "), Span::raw(crate::t!("tui.hint.palette").to_string())]),
        Line::from(vec![Span::styled("  ↑/↓", Style::default().fg(theme.accent)), Span::raw("        "), Span::raw(crate::t!("tui.hint.scroll").to_string())]),
        Line::from(vec![Span::styled("  s", Style::default().fg(theme.accent)), Span::raw("            "), Span::raw(crate::t!("tui.hint.sidebar").to_string())]),
        Line::from(vec![Span::styled("  q", Style::default().fg(theme.accent)), Span::raw("            "), Span::raw(crate::t!("tui.hint.quit").to_string())]),
        Line::from(vec![Span::styled("  Ctrl+C", Style::default().fg(theme.error)), Span::raw("      "), Span::raw(crate::t!("tui.hint.force_quit").to_string())]),
        Line::from(""),
        Line::from(Span::styled(crate::t!("tui.help.close").to_string(), Style::default().fg(theme.text_muted))),
    ];
    let pa = centered_rect(50, 60, area);
    f.render_widget(Clear, pa);
    let block = Block::default().borders(Borders::ALL).border_style(Style::default().fg(theme.border_active))
        .title(Span::styled(format!(" {} ", crate::t!("tui.help.title")), Style::default().fg(theme.primary)))
        .style(Style::default().bg(theme.background_element));
    f.render_widget(block, pa);
    f.render_widget(Paragraph::new(lines), Rect { x: pa.x + 1, y: pa.y + 1, width: pa.width.saturating_sub(2), height: pa.height.saturating_sub(2) });
}

fn centered_rect(px: u16, py: u16, area: Rect) -> Rect {
    let vl = Layout::default().direction(Direction::Vertical).constraints([Constraint::Percentage((100-py)/2), Constraint::Percentage(py), Constraint::Percentage((100-py)/2)]).split(area);
    Layout::default().direction(Direction::Horizontal).constraints([Constraint::Percentage((100-px)/2), Constraint::Percentage(px), Constraint::Percentage((100-px)/2)]).split(vl[1])[1]
}

/// Word-wrap `text` into a list of plain strings, each at most `max_w` chars.
/// Whitespace runs are collapsed to single spaces. Empty input yields a
/// single empty string (so callers always have at least one line to render).
fn wrap_words(text: &str, max_w: usize) -> Vec<String> {
    let max_w = max_w.max(1);
    let mut result: Vec<String> = Vec::new();
    let mut cur = String::new();
    for word in text.split_whitespace() {
        if cur.len() + word.len() + 1 > max_w && !cur.is_empty() {
            result.push(std::mem::take(&mut cur));
        }
        if !cur.is_empty() {
            cur.push(' ');
        }
        cur.push_str(word);
    }
    if !cur.is_empty() || result.is_empty() {
        result.push(cur);
    }
    result
}
