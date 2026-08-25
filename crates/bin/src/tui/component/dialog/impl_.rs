use super::*;
impl Dialog {
    // -- Constructors --

    pub fn alert(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            kind: DialogKind::Alert,
            title: title.into(),
            message: message.into(),
            options: vec![],
            selected: 0,
            filter: String::new(),
            visible: true,
            size: DialogSize::Medium,
            confirm_focus: ConfirmFocus::Confirm,
            help_text: String::new(),
            locked: false,
            created: Instant::now(),
            scroll_offset: 0,
        }
    }

    pub fn confirm(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            kind: DialogKind::Confirm,
            title: title.into(),
            message: message.into(),
            options: vec![],
            selected: 0,
            filter: String::new(),
            visible: true,
            size: DialogSize::Medium,
            confirm_focus: ConfirmFocus::Confirm,
            help_text: String::new(),
            locked: false,
            created: Instant::now(),
            scroll_offset: 0,
        }
    }

    pub fn select(title: impl Into<String>, options: Vec<DialogOption>) -> Self {
        Self {
            kind: DialogKind::Select,
            title: title.into(),
            message: String::new(),
            options,
            selected: 0,
            filter: String::new(),
            visible: true,
            size: DialogSize::Medium,
            confirm_focus: ConfirmFocus::Confirm,
            help_text: String::new(),
            locked: false,
            created: Instant::now(),
            scroll_offset: 0,
        }
    }

    pub fn help(help_text: impl Into<String>) -> Self {
        Self {
            kind: DialogKind::Help,
            title: crate::t!("tui.dialog.help").to_string(),
            message: String::new(),
            options: vec![],
            selected: 0,
            filter: String::new(),
            visible: true,
            size: DialogSize::Medium,
            confirm_focus: ConfirmFocus::Confirm,
            help_text: help_text.into(),
            locked: false,
            created: Instant::now(),
            scroll_offset: 0,
        }
    }

    // -- Size --

    pub fn set_size(&mut self, size: DialogSize) {
        self.size = size;
    }

    // -- Visibility --

    pub fn close(&mut self) {
        self.visible = false;
    }

    pub fn replace(&mut self, kind: DialogKind, title: impl Into<String>, message: impl Into<String>) {
        self.kind = kind;
        self.title = title.into();
        self.message = message.into();
        self.selected = 0;
        self.filter.clear();
        self.visible = true;
        self.size = DialogSize::Medium;
        self.scroll_offset = 0;
        self.confirm_focus = ConfirmFocus::Confirm;
        self.created = Instant::now();
    }

    // -- Filtering --

    /// Returns filtered options based on the current filter string.
    ///
    /// Mirrors the `filtered()` memo in dialog-select.tsx:
    /// - If `skip_filter` is false (default), uses case-insensitive substring match on title.
    /// - Disabled options are always excluded.
    pub fn filtered_options(&self) -> Vec<&DialogOption> {
        if self.filter.is_empty() {
            return self.options.iter().filter(|o| !o.disabled).collect();
        }
        let needle = self.filter.to_lowercase();
        self.options
            .iter()
            .filter(|o| !o.disabled)
            .filter(|o| {
                o.title.to_lowercase().contains(&needle)
                    || o.category.as_ref().map(|c| c.to_lowercase().contains(&needle)).unwrap_or(false)
            })
            .collect()
    }

    /// Grouped filtered options — (category, options) pairs, preserving order.
    pub fn grouped_filtered(&self) -> Vec<(String, Vec<&DialogOption>)> {
        let filtered = self.filtered_options();
        let mut groups: Vec<(String, Vec<&DialogOption>)> = Vec::new();
        for opt in &filtered {
            let cat = opt.category.clone().unwrap_or_default();
            if let Some(last) = groups.last_mut() {
                if last.0 == cat {
                    last.1.push(*opt);
                    continue;
                }
            }
            groups.push((cat, vec![*opt]));
        }
        groups
    }

    // -- Key handling --

    pub fn handle_key(&mut self, key: KeyEvent) -> DialogResult {
        // Ctrl+C always closes — matches the TS binding.
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return DialogResult::Close;
        }

        match self.kind {
            DialogKind::Alert => self.handle_alert_key(key),
            DialogKind::Confirm => self.handle_confirm_key(key),
            DialogKind::Select => self.handle_select_key(key),
            DialogKind::Help => self.handle_help_key(key),
        }
    }

    pub fn handle_alert_key(&mut self, key: KeyEvent) -> DialogResult {
        match key.code {
            KeyCode::Enter | KeyCode::Esc => DialogResult::Close,
            _ => DialogResult::None,
        }
    }

    pub fn handle_confirm_key(&mut self, key: KeyEvent) -> DialogResult {
        match key.code {
            KeyCode::Enter => {
                let result = match self.confirm_focus {
                    ConfirmFocus::Confirm => DialogResult::Confirm,
                    ConfirmFocus::Cancel => DialogResult::Cancel,
                };
                result
            }
            KeyCode::Left | KeyCode::Char('h') => {
                self.confirm_focus = self.confirm_focus.toggle();
                DialogResult::None
            }
            KeyCode::Right | KeyCode::Char('l') => {
                self.confirm_focus = self.confirm_focus.toggle();
                DialogResult::None
            }
            KeyCode::Esc => DialogResult::Close,
            _ => DialogResult::None,
        }
    }

    pub fn handle_help_key(&mut self, key: KeyEvent) -> DialogResult {
        match key.code {
            KeyCode::Enter | KeyCode::Esc => DialogResult::Close,
            _ => DialogResult::None,
        }
    }

    pub fn handle_select_key(&mut self, key: KeyEvent) -> DialogResult {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                if self.locked { return DialogResult::None; }
                let max = self.filtered_options().len();
                if max == 0 { return DialogResult::None; }
                if self.selected == 0 {
                    self.selected = max - 1;
                } else {
                    self.selected -= 1;
                }
                self.clamp_scroll();
                DialogResult::None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.locked { return DialogResult::None; }
                let max = self.filtered_options().len().saturating_sub(1);
                if self.selected >= max {
                    self.selected = 0;
                } else {
                    self.selected += 1;
                }
                self.clamp_scroll();
                DialogResult::None
            }
            KeyCode::PageUp => {
                if self.locked { return DialogResult::None; }
                self.selected = self.selected.saturating_sub(10);
                self.clamp_scroll();
                DialogResult::None
            }
            KeyCode::PageDown => {
                if self.locked { return DialogResult::None; }
                let max = self.filtered_options().len().saturating_sub(1);
                self.selected = (self.selected + 10).min(max);
                self.clamp_scroll();
                DialogResult::None
            }
            KeyCode::Home => {
                if self.locked { return DialogResult::None; }
                self.selected = 0;
                self.scroll_offset = 0;
                DialogResult::None
            }
            KeyCode::End => {
                if self.locked { return DialogResult::None; }
                self.selected = self.filtered_options().len().saturating_sub(1);
                self.clamp_scroll();
                DialogResult::None
            }
            KeyCode::Enter => {
                let opts = self.filtered_options();
                if let Some(opt) = opts.get(self.selected) {
                    DialogResult::Select(opt.value.clone())
                } else {
                    DialogResult::Close
                }
            }
            KeyCode::Esc => DialogResult::Close,
            KeyCode::Backspace => {
                if self.locked { return DialogResult::None; }
                self.filter.pop();
                self.selected = 0;
                self.scroll_offset = 0;
                DialogResult::None
            }
            KeyCode::Char(c) => {
                if self.locked { return DialogResult::None; }
                if c.is_control() { return DialogResult::None; }
                self.filter.push(c);
                self.selected = 0;
                self.scroll_offset = 0;
                DialogResult::None
            }
            _ => DialogResult::None,
        }
    }

    /// Ensure the selected item is visible within the scroll viewport.
    pub fn clamp_scroll(&mut self) {
        // Keep selected within view; the actual viewport height is set during render.
        // We keep scroll_offset updated so that selected is always visible.
        let visible_height: usize = 10; // approximate; render will adjust
        if self.selected < self.scroll_offset {
            self.scroll_offset = self.selected;
        } else if self.selected >= self.scroll_offset + visible_height {
            self.scroll_offset = self.selected.saturating_sub(visible_height - 1);
        }
    }

    // -- Rendering --

    /// Render the dialog as a modal overlay covering the full `area`.
    ///
    /// Mirrors the TS `Dialog` component:
    /// - Full-screen overlay with semi-transparent black background.
    /// - Centered panel with `theme.backgroundPanel`.
    /// - Width based on `size`.
    /// - Content area below a padding-top of `height/4`.
    pub fn render(&self, f: &mut Frame, area: Rect, theme: &Theme) {
        if !self.visible { return; }

        // --- Semi-transparent backdrop ---
        // ratatui doesn't support alpha, so we approximate with Clear + a dark fill.
        f.render_widget(Clear, area);
        let backdrop = Block::default().style(Style::default().bg(Color::Rgb(0, 0, 0)));
        f.render_widget(backdrop, area);

        // --- Dialog panel ---
        let dialog_width = std::cmp::min(self.size.width(), area.width.saturating_sub(2));
        let popup_area = centered_rect(dialog_width, area);

        f.render_widget(Clear, popup_area);

        let panel = Block::default()
            .borders(Borders::NONE)
            .style(Style::default().bg(theme.background_panel));
        f.render_widget(panel, popup_area);

        // Inner content area with padding (paddingTop: 1 in TS).
        let inner = Rect {
            x: popup_area.x + 1,
            y: popup_area.y + 1,
            width: popup_area.width.saturating_sub(2),
            height: popup_area.height.saturating_sub(2),
        };

        match self.kind {
            DialogKind::Alert => self.render_alert(f, inner, theme),
            DialogKind::Confirm => self.render_confirm(f, inner, theme),
            DialogKind::Select => self.render_select(f, inner, theme),
            DialogKind::Help => self.render_help(f, inner, theme),
        }
    }

    // -- Alert rendering --

    /// Mirrors `DialogAlert` — title row + message + OK button.
    pub fn render_alert(&self, f: &mut Frame, area: Rect, theme: &Theme) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // title row
                Constraint::Length(1), // gap
                Constraint::Min(1),   // message
                Constraint::Length(1), // gap
                Constraint::Length(1), // OK button
            ])
            .split(area);

        // Title + esc hint
        let title_line = Line::from(vec![
            Span::styled(self.title.clone(), Style::default().fg(theme.text).add_modifier(Modifier::BOLD)),
            Span::raw("  "),
            Span::styled(crate::t!("tui.prompt.esc").to_string(), Style::default().fg(theme.text_muted)),
        ]);
        f.render_widget(Paragraph::new(title_line), chunks[0]);

        // Message
        let msg = Paragraph::new(Line::from(Span::styled(
            self.message.clone(),
            Style::default().fg(theme.text_muted),
        )));
        f.render_widget(msg, chunks[2]);

        // OK button (right-aligned)
        let ok_area = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(0), Constraint::Length(6)])
            .split(chunks[4])[1];

        let ok_block = Block::default()
            .borders(Borders::NONE)
            .style(Style::default().bg(theme.primary));
        f.render_widget(ok_block, ok_area);

        let ok_text = Paragraph::new(Line::from(Span::styled(
            format!("  {}  ", crate::t!("tui.dialog.ok")),
            Style::default().fg(theme.text),
        )))
        .alignment(Alignment::Center);
        f.render_widget(ok_text, ok_area);
    }

    // -- Confirm rendering --

    /// Mirrors `DialogConfirm` — title + message + Cancel/Confirm buttons.
    pub fn render_confirm(&self, f: &mut Frame, area: Rect, theme: &Theme) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // title row
                Constraint::Length(1), // gap
                Constraint::Min(1),    // message
                Constraint::Length(1), // gap
                Constraint::Length(1), // buttons
            ])
            .split(area);

        // Title + esc
        let title_line = Line::from(vec![
            Span::styled(self.title.clone(), Style::default().fg(theme.text).add_modifier(Modifier::BOLD)),
            Span::raw("  "),
            Span::styled(crate::t!("tui.prompt.esc").to_string(), Style::default().fg(theme.text_muted)),
        ]);
        f.render_widget(Paragraph::new(title_line), chunks[0]);

        // Message
        let msg = Paragraph::new(Line::from(Span::styled(
            self.message.clone(),
            Style::default().fg(theme.text_muted),
        )));
        f.render_widget(msg, chunks[2]);

        // Buttons: cancel | confirm (right-aligned)
        let btn_row = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Min(0),
                Constraint::Length(10), // cancel
                Constraint::Length(1),  // gap
                Constraint::Length(12), // confirm
            ])
            .split(chunks[4]);

        let cancel_active = self.confirm_focus == ConfirmFocus::Cancel;
        let confirm_active = self.confirm_focus == ConfirmFocus::Confirm;

        let cancel_style = if cancel_active {
            Style::default().bg(theme.primary).fg(theme.text)
        } else {
            Style::default().fg(theme.text_muted)
        };
        let confirm_style = if confirm_active {
            Style::default().bg(theme.primary).fg(theme.text)
        } else {
            Style::default().fg(theme.text_muted)
        };

        let cancel_para = Paragraph::new(Line::from(Span::styled(
            format!("  {}  ", crate::t!("tui.dialog.cancel")),
            cancel_style,
        )));
        f.render_widget(cancel_para, btn_row[1]);

        let confirm_para = Paragraph::new(Line::from(Span::styled(
            format!("  {}  ", crate::t!("tui.dialog.confirm")),
            confirm_style,
        )));
        f.render_widget(confirm_para, btn_row[3]);
    }

    // -- Help rendering --

    /// Mirrors `DialogHelp` — title + help text + OK button.
    pub fn render_help(&self, f: &mut Frame, area: Rect, theme: &Theme) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // title
                Constraint::Length(1), // gap
                Constraint::Min(1),    // help text
                Constraint::Length(1), // gap
                Constraint::Length(1), // OK button
            ])
            .split(area);

        let title_line = Line::from(vec![
            Span::styled(crate::t!("tui.dialog.help").to_string(), Style::default().fg(theme.text).add_modifier(Modifier::BOLD)),
            Span::raw("  "),
            Span::styled(crate::t!("tui.dialog.esc_enter").to_string(), Style::default().fg(theme.text_muted)),
        ]);
        f.render_widget(Paragraph::new(title_line), chunks[0]);

        let help = Paragraph::new(Line::from(Span::styled(
            self.help_text.clone(),
            Style::default().fg(theme.text_muted),
        )))
        .wrap(Wrap { trim: true });
        f.render_widget(help, chunks[2]);

        // OK button
        let ok_area = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(0), Constraint::Length(6)])
            .split(chunks[4])[1];

        let ok_block = Block::default().style(Style::default().bg(theme.primary));
        f.render_widget(ok_block, ok_area);
        f.render_widget(
            Paragraph::new(Line::from(Span::styled(format!("  {}  ", crate::t!("tui.dialog.ok")), Style::default().fg(theme.text)))).alignment(Alignment::Center),
            ok_area,
        );
    }

    // -- Select rendering --

    /// Mirrors `DialogSelect` — title + filter input + scrollable option list.
    pub fn render_select(&self, f: &mut Frame, area: Rect, theme: &Theme) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // title row
                Constraint::Length(1), // gap
                Constraint::Length(1), // filter input
                Constraint::Length(1), // gap
                Constraint::Min(1),    // option list
            ])
            .split(area);

        // Title + esc hint
        let title_line = Line::from(vec![
            Span::styled(self.title.clone(), Style::default().fg(theme.text).add_modifier(Modifier::BOLD)),
            Span::raw("  "),
            Span::styled(crate::t!("tui.prompt.esc").to_string(), Style::default().fg(theme.text_muted)),
        ]);
        f.render_widget(Paragraph::new(title_line), chunks[0]);

        // Filter input / placeholder
        let filter_line = if self.filter.is_empty() {
            Line::from(Span::styled(crate::t!("tui.dialog.search").to_string(), Style::default().fg(theme.text_muted)))
        } else {
            Line::from(Span::styled(
                format!("> {}", self.filter),
                Style::default().fg(theme.accent),
            ))
        };
        f.render_widget(Paragraph::new(filter_line), chunks[2]);

        // Option list
        let opts = self.filtered_options();
        if opts.is_empty() {
            let empty = Paragraph::new(Line::from(Span::styled(
                crate::t!("tui.dialog.no_results").to_string(),
                Style::default().fg(theme.text_muted),
            )));
            f.render_widget(empty, chunks[4]);
            return;
        }

        let list_area = chunks[4];
        let visible_height = list_area.height as usize;

        // Adjust scroll_offset to keep selected visible.
        let mut scroll = self.scroll_offset;
        if self.selected < scroll {
            scroll = self.selected;
        } else if self.selected >= scroll + visible_height {
            scroll = self.selected.saturating_sub(visible_height.saturating_sub(1));
        }

        let visible_opts: Vec<&DialogOption> = opts
            .iter()
            .skip(scroll)
            .take(visible_height)
            .cloned()
            .collect();

        let items: Vec<ListItem> = visible_opts
            .iter()
            .enumerate()
            .map(|(i, o)| {
                let real_index = scroll + i;
                let is_selected = real_index == self.selected;
                let style = if is_selected {
                    Style::default().bg(theme.primary).fg(theme.text).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(theme.text)
                };

                let mut spans: Vec<Span> = Vec::new();

                // Category header if changed
                if let Some(ref cat) = o.category {
                    if i == 0 || visible_opts.get(i.wrapping_sub(1)).and_then(|p| p.category.as_ref()) != Some(cat) {
                        spans.push(Span::styled(
                            format!("{} ", cat),
                            Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
                        ));
                    }
                }

                // Current marker (●)
                if is_selected {
                    spans.push(Span::styled("● ", style));
                } else {
                    spans.push(Span::raw("  "));
                }

                // Title
                spans.push(Span::styled(o.title.clone(), style));

                // Description
                if let Some(ref desc) = o.description {
                    if Some(desc.as_str()) != o.category.as_deref() {
                        spans.push(Span::styled(format!(" {}", desc), Style::default().fg(theme.text_muted)));
                    }
                }

                // Details
                for detail in &o.details {
                    spans.push(Span::raw("\n  "));
                    spans.push(Span::styled(detail.clone(), Style::default().fg(theme.text_muted)));
                }

                ListItem::new(Text::from(Line::from(spans)))
            })
            .collect();

        let list = List::new(items);
        f.render_widget(list, list_area);

        // Render scroll indicator
        if opts.len() > visible_height {
            let scroll_info = format!(" {}/{} ", self.selected + 1, opts.len());
            let scroll_para = Paragraph::new(Line::from(Span::styled(
                scroll_info,
                Style::default().fg(theme.text_muted),
            )))
            .alignment(Alignment::Right);
            let scroll_area = Rect {
                x: list_area.x,
                y: list_area.bottom().saturating_sub(1),
                width: list_area.width,
                height: 1,
            };
            f.render_widget(scroll_para, scroll_area);
        }
    }
}

