use super::*;

impl Prompt {
    // -- Rendering -----------------------------------------------------------

    /// Render the prompt component.
    ///
    /// Replicates the original opencode TUI Prompt layout:
    /// ```text
    /// ┃  placeholder text              <- textarea (bg=backgroundElement)
    /// ┃  Build · model_name Provider   <- agent/model meta line
    /// ╹▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀    <- separator (╹ in border color, ▀ in bgElement)
    /// tab agents  ctrl+p commands      <- hint line
    /// ```
    pub fn render(&self, f: &mut Frame, area: Rect, theme: &Theme) {
        let border_color = if self.mode == PromptMode::Shell {
            theme.primary
        } else if self.focused {
            theme.border_active
        } else {
            theme.border
        };

        // Layout: main area (text + meta) + separator line + hint line.
        // Matches the TS Prompt: left border ┃ + background_element fill.
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(3),    // 0: main (text + meta)
                Constraint::Length(1), // 1: ╹▀▀▀ separator
                Constraint::Length(1), // 2: hint row
            ])
            .split(area);

        let main_area = chunks[0];
        let sep_area = chunks[1];
        let hint_area = chunks[2];
        let bg = theme.background_element;

        // ┃ left border column (full height of main_area).
        let border_area = Rect {
            x: main_area.x,
            y: main_area.y,
            width: 1,
            height: main_area.height,
        };
        let border_lines: Vec<Line> = (0..main_area.height)
            .map(|_| Line::from(Span::styled("┃", Style::default().fg(border_color).bg(bg))))
            .collect();
        f.render_widget(Paragraph::new(border_lines), border_area);

        // Inner content area (right of the ┃ border).
        let inner = Rect {
            x: main_area.x + 1,
            y: main_area.y,
            width: main_area.width.saturating_sub(1),
            height: main_area.height,
        };
        // Fill the inner area with background_element so the whole box reads
        // as a single panel.
        f.render_widget(Paragraph::new("").style(Style::default().bg(bg)), inner);

        // Text area: all but the last line (meta).
        let text_area = Rect {
            x: inner.x + 1,
            y: inner.y + 1,
            width: inner.width.saturating_sub(2),
            height: inner.height.saturating_sub(2),
        };

        let meta_area = Rect {
            x: inner.x + 1,
            y: inner.y + inner.height.saturating_sub(1),
            width: inner.width.saturating_sub(2),
            height: 1,
        };

        let display_text = if self.input.is_empty() {
            self.placeholder_text()
        } else {
            self.input.clone()
        };
        let text_style = if self.input.is_empty() {
            Style::default().fg(theme.text_muted).bg(bg)
        } else {
            Style::default().fg(theme.text).bg(bg)
        };
        let lines: Vec<Line> = display_text
            .lines()
            .map(|l| Line::from(Span::styled(l.to_string(), text_style)))
            .collect();
        f.render_widget(
            Paragraph::new(lines)
                .style(Style::default().bg(bg))
                .wrap(Wrap { trim: false }),
            text_area,
        );

        // Meta line: agent · model · provider (matches TS).
        let meta_spans: Vec<Span> = match self.mode {
            PromptMode::Shell => vec![
                Span::styled("Shell", Style::default().fg(theme.primary).bg(bg)),
            ],
            PromptMode::Normal => {
                let mut spans = vec![
                    Span::styled(self.agent.clone(), Style::default().fg(theme.primary).bg(bg)),
                ];
                if !self.model.is_empty() {
                    spans.push(Span::styled(" · ", Style::default().fg(theme.text_muted).bg(bg)));
                    spans.push(Span::styled(self.model.clone(), Style::default().fg(theme.text).bg(bg)));
                    if !self.provider.is_empty() {
                        spans.push(Span::styled(" · ", Style::default().fg(theme.text_muted).bg(bg)));
                        spans.push(Span::styled(self.provider.clone(), Style::default().fg(theme.text_muted).bg(bg)));
                    }
                } else {
                    spans.push(Span::styled(" · ", Style::default().fg(theme.text_muted).bg(bg)));
                    spans.push(Span::styled(crate::t!("tui.prompt.no_model").to_string(), Style::default().fg(theme.text_muted).bg(bg)));
                }
                spans
            }
        };
        f.render_widget(Paragraph::new(Line::from(meta_spans)), meta_area);

        // Separator: ╹ + ▀ fill (matches TS bottom border).
        let sep_width = sep_area.width.saturating_sub(1) as usize;
        let sep_spans = vec![
            Span::styled("╹", Style::default().fg(border_color)),
            Span::styled("▀".repeat(sep_width), Style::default().fg(bg)),
        ];
        f.render_widget(Paragraph::new(Line::from(sep_spans)), sep_area);

        // Hint row: left (status / cwd) + right (usage · commands · OpenCode).
        // We left-align the hint text; the right side is rendered by the caller
        // (home/session) if needed, or by the footer.
        let hint_spans: Vec<Span> = match self.mode {
            PromptMode::Shell => vec![
                Span::styled("esc ", Style::default().fg(theme.text)),
                Span::styled(crate::t!("tui.prompt.hint_esc_shell").to_string(), Style::default().fg(theme.text_muted)),
            ],
            PromptMode::Normal => vec![
                Span::styled("tab ", Style::default().fg(theme.text)),
                Span::styled(crate::t!("tui.prompt.hint_agents").to_string(), Style::default().fg(theme.text_muted)),
                Span::raw("   "),
                Span::styled("ctrl+p ", Style::default().fg(theme.text)),
                Span::styled(crate::t!("tui.prompt.hint_commands").to_string(), Style::default().fg(theme.text_muted)),
            ],
        };
        f.render_widget(Paragraph::new(Line::from(hint_spans)), hint_area);

        if self.focused {
            let pos = self.cursor_pos();
            // Use display width (unicode-width) so wide chars like CJK
            // (width=2) don't misalign the cursor.
            let line: String = self
                .input
                .lines()
                .nth(pos.row)
                .unwrap_or("")
                .chars()
                .take(pos.col)
                .collect();
            let display_width = unicode_width::UnicodeWidthStr::width(line.as_str()) as u16;
            let cursor_x = text_area.x + display_width;
            let cursor_y = text_area.y + pos.row as u16;
            let cursor_x = cursor_x.min(text_area.x + text_area.width.saturating_sub(1));
            let cursor_y = cursor_y.min(text_area.y + text_area.height.saturating_sub(1));
            f.set_cursor_position((cursor_x, cursor_y));
        }
    }
}
