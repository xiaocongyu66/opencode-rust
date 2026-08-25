//! Question prompt — modal for question/answer interaction.
//! Ported from opencode/packages/tui/src/routes/session/question.tsx (515 lines)
//!
//! Features:
//! - Single-select: pick one option and auto-submit
//! - Multi-select: toggle options, then go to Confirm tab
//! - Multiple questions with tab navigation
//! - Custom "Type your own answer" option with text input
//! - Keyboard: 1-9 select, j/k navigate, Tab next question, Enter confirm

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;
use crate::tui::theme::Theme;

// ---------------------------------------------------------------------------
// Question types
// ---------------------------------------------------------------------------

/// A single question option.
#[derive(Debug, Clone)]
pub struct QuestionOption {
    pub label: String,
    pub description: Option<String>,
}

/// A single question.
#[derive(Debug, Clone)]
pub struct QuestionItem {
    pub header: String,
    pub question: String,
    pub options: Vec<QuestionOption>,
    pub multiple: bool,
    pub custom: bool,
}

/// A question request from the server.
#[derive(Debug, Clone)]
pub struct QuestionRequest {
    pub id: String,
    pub session_id: String,
    pub questions: Vec<QuestionItem>,
}

// ---------------------------------------------------------------------------
// Question result
// ---------------------------------------------------------------------------

/// Result of processing a key in the question prompt.
#[derive(Debug)]
pub enum QuestionResult {
    None,
    /// User submitted answers.
    Submit(Vec<Vec<String>>),
    /// User rejected the question.
    Reject,
}

// ---------------------------------------------------------------------------
// Question state — holds UI state for the active question prompt
// ---------------------------------------------------------------------------

pub struct QuestionState {
    pub request: QuestionRequest,
    pub tab: usize,
    pub selected: usize,
    pub answers: Vec<Vec<String>>,
    pub custom_text: Vec<String>,
    pub editing: bool,
}

impl QuestionState {
    pub fn new(request: QuestionRequest) -> Self {
        let qcount = request.questions.len();
        Self {
            request,
            tab: 0,
            selected: 0,
            answers: vec![Vec::new(); qcount],
            custom_text: vec![String::new(); qcount],
            editing: false,
        }
    }

    // -----------------------------------------------------------------------
    // Computed properties
    // -----------------------------------------------------------------------

    fn questions(&self) -> &[QuestionItem] {
        &self.request.questions
    }

    fn single(&self) -> bool {
        self.questions().len() == 1 && !self.questions()[0].multiple
    }

    fn tabs(&self) -> usize {
        if self.single() {
            1
        } else {
            self.questions().len() + 1
        }
    }

    fn current_question(&self) -> Option<&QuestionItem> {
        self.questions().get(self.tab)
    }

    fn is_confirm_tab(&self) -> bool {
        !self.single() && self.tab == self.questions().len()
    }

    fn options_count(&self) -> usize {
        let q = match self.current_question() {
            Some(q) => q,
            None => return 0,
        };
        let extra = if q.custom { 1 } else { 0 };
        q.options.len() + extra
    }

    fn is_other_selected(&self) -> bool {
        let q = match self.current_question() {
            Some(q) => q,
            None => return false,
        };
        q.custom && self.selected == q.options.len()
    }

    // -----------------------------------------------------------------------
    // Key handling
    // -----------------------------------------------------------------------

    pub fn handle_key(&mut self, key: KeyEvent) -> QuestionResult {
        if self.editing && !self.is_confirm_tab() {
            return self.handle_editing_key(key);
        }
        self.handle_normal_key(key)
    }

    fn handle_editing_key(&mut self, key: KeyEvent) -> QuestionResult {
        match key.code {
            KeyCode::Esc => {
                self.editing = false;
                QuestionResult::None
            }
            KeyCode::Enter => {
                let text = self.custom_text.get(self.tab).cloned().unwrap_or_default();
                let text = text.trim().to_string();
                if text.is_empty() {
                    if let Some(prev) = self.custom_text.get(self.tab).cloned() {
                        if !prev.is_empty() {
                            if let Some(slot) = self.custom_text.get_mut(self.tab) {
                                slot.clear();
                            }
                            if let Some(answers) = self.answers.get_mut(self.tab) {
                                answers.retain(|x| x != &prev);
                            }
                        }
                    }
                    self.editing = false;
                    return QuestionResult::None;
                }
                let q = match self.current_question() {
                    Some(q) => q,
                    None => return QuestionResult::None,
                };
                if q.multiple {
                    if let Some(slot) = self.custom_text.get_mut(self.tab) {
                        *slot = text.clone();
                    }
                    if let Some(answers) = self.answers.get_mut(self.tab) {
                        let prev = self.custom_text.get(self.tab).cloned().unwrap_or_default();
                        if !prev.is_empty() {
                            answers.retain(|x| x != &prev);
                        }
                        if !answers.contains(&text) {
                            answers.push(text);
                        }
                    }
                    self.editing = false;
                    QuestionResult::None
                } else {
                    self.pick(text.clone(), true);
                    self.editing = false;
                    QuestionResult::None
                }
            }
            KeyCode::Backspace => {
                if let Some(slot) = self.custom_text.get_mut(self.tab) {
                    if !slot.is_empty() {
                        let prev = slot[..slot.len()]
                            .char_indices()
                            .last()
                            .map(|(i, _)| i)
                            .unwrap_or(0);
                        slot.replace_range(prev.., "");
                    }
                }
                QuestionResult::None
            }
            KeyCode::Char(c) => {
                if !key.modifiers.contains(KeyModifiers::CONTROL) {
                    if let Some(slot) = self.custom_text.get_mut(self.tab) {
                        slot.push(c);
                    }
                }
                QuestionResult::None
            }
            _ => QuestionResult::None,
        }
    }

    fn handle_normal_key(&mut self, key: KeyEvent) -> QuestionResult {
        let tabs = self.tabs();

        match key.code {
            // Tab navigation
            KeyCode::Left | KeyCode::Char('h') => {
                self.tab = (self.tab + tabs - 1) % tabs;
                self.selected = 0;
                QuestionResult::None
            }
            KeyCode::Right | KeyCode::Char('l') => {
                self.tab = (self.tab + 1) % tabs;
                self.selected = 0;
                QuestionResult::None
            }
            KeyCode::Tab => {
                let delta = if key.modifiers.contains(KeyModifiers::SHIFT) { tabs - 1 } else { 1 };
                self.tab = (self.tab + delta) % tabs;
                self.selected = 0;
                QuestionResult::None
            }

            // Confirm tab
            KeyCode::Enter if self.is_confirm_tab() => {
                let answers = self.answers.clone();
                QuestionResult::Submit(answers)
            }
            KeyCode::Esc if self.is_confirm_tab() => {
                QuestionResult::Reject
            }

            // Number keys 1-9
            KeyCode::Char(c) if c.is_ascii_digit() && !self.is_confirm_tab() => {
                let n = c as usize - '0' as usize;
                let total = self.options_count();
                if n >= 1 && n <= total.min(9) {
                    self.selected = n - 1;
                    self.select_option();
                }
                QuestionResult::None
            }

            // Navigation
            KeyCode::Up | KeyCode::Char('k') if !self.is_confirm_tab() => {
                let total = self.options_count();
                if total > 0 {
                    self.selected = (self.selected + total - 1) % total;
                }
                QuestionResult::None
            }
            KeyCode::Down | KeyCode::Char('j') if !self.is_confirm_tab() => {
                let total = self.options_count();
                if total > 0 {
                    self.selected = (self.selected + 1) % total;
                }
                QuestionResult::None
            }

            // Select
            KeyCode::Enter if !self.is_confirm_tab() => {
                self.select_option();
                QuestionResult::None
            }

            KeyCode::Esc => QuestionResult::Reject,

            _ => QuestionResult::None,
        }
    }

    // -----------------------------------------------------------------------
    // Selection logic
    // -----------------------------------------------------------------------

    fn select_option(&mut self) {
        if self.is_other_selected() {
            let q = match self.current_question() {
                Some(q) => q,
                None => return,
            };
            if !q.multiple {
                self.editing = true;
                return;
            }
            let text = self.custom_text.get(self.tab).cloned().unwrap_or_default();
            if !text.is_empty() && self.custom_picked() {
                self.toggle(text);
                return;
            }
            self.editing = true;
            return;
        }
        let q = match self.current_question() {
            Some(q) => q,
            None => return,
        };
        let opt = match q.options.get(self.selected) {
            Some(o) => o,
            None => return,
        };
        if q.multiple {
            self.toggle(opt.label.clone());
        } else {
            self.pick(opt.label.clone(), false);
        }
    }

    fn pick(&mut self, answer: String, custom: bool) {
        if let Some(slot) = self.answers.get_mut(self.tab) {
            slot.clear();
            slot.push(answer.clone());
        }
        if custom {
            if let Some(slot) = self.custom_text.get_mut(self.tab) {
                *slot = answer;
            }
        }
        if self.single() {
            return;
        }
        self.tab += 1;
        self.selected = 0;
    }

    fn toggle(&mut self, answer: String) {
        if let Some(answers) = self.answers.get_mut(self.tab) {
            if let Some(idx) = answers.iter().position(|x| x == &answer) {
                answers.remove(idx);
            } else {
                answers.push(answer);
            }
        }
    }

    fn custom_picked(&self) -> bool {
        let text = self.custom_text.get(self.tab).cloned().unwrap_or_default();
        if text.is_empty() {
            return false;
        }
        self.answers.get(self.tab)
            .map(|a| a.contains(&text))
            .unwrap_or(false)
    }

    // -----------------------------------------------------------------------
    // Rendering
    // -----------------------------------------------------------------------

    pub fn render(&self, f: &mut Frame, area: Rect, theme: &Theme) {
        let mut lines: Vec<Line> = Vec::new();

        // Tab bar (only for multi-question)
        if !self.single() {
            let mut tab_spans: Vec<Span> = Vec::new();
            for (i, q) in self.questions().iter().enumerate() {
                let is_active = i == self.tab;
                let is_answered = self.answers.get(i).map(|a| !a.is_empty()).unwrap_or(false);
                let (bg, fg) = if is_active {
                    (theme.accent, theme.background)
                } else {
                    (theme.background_panel, if is_answered { theme.text } else { theme.text_muted })
                };
                tab_spans.push(Span::styled(
                    format!(" {} ", q.header),
                    Style::default().fg(fg).bg(bg),
                ));
                tab_spans.push(Span::raw(" "));
            }
            // Confirm tab
            let confirm_active = self.is_confirm_tab();
            let (bg, fg) = if confirm_active {
                (theme.accent, theme.background)
            } else {
                (theme.background_panel, theme.text_muted)
            };
            tab_spans.push(Span::styled(
                " Confirm ",
                Style::default().fg(fg).bg(bg),
            ));
            lines.push(Line::from(tab_spans));
            lines.push(Line::from(""));
        }

        if self.is_confirm_tab() {
            // Review tab
            lines.push(Line::from(Span::styled(
                "Review",
                Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(""));
            for (i, q) in self.questions().iter().enumerate() {
                let value = self.answers.get(i)
                    .map(|a| a.join(", "))
                    .unwrap_or_default();
                let answered = !value.is_empty();
                lines.push(Line::from(vec![
                    Span::styled(format!("  {}: ", q.header), Style::default().fg(theme.text_muted)),
                    Span::styled(
                        if answered { value } else { "(not answered)".to_string() },
                        Style::default().fg(if answered { theme.text } else { theme.error }),
                    ),
                ]));
            }
        } else if let Some(q) = self.current_question() {
            // Question text
            let suffix = if q.multiple { " (select all that apply)" } else { "" };
            lines.push(Line::from(Span::styled(
                format!("  {}{}", q.question, suffix),
                Style::default().fg(theme.text),
            )));
            lines.push(Line::from(""));

            // Options
            for (i, opt) in q.options.iter().enumerate() {
                let active = i == self.selected;
                let picked = self.answers.get(self.tab)
                    .map(|a| a.contains(&opt.label))
                    .unwrap_or(false);
                let num_style = if active {
                    Style::default().fg(theme.text_muted)
                } else {
                    Style::default().fg(theme.text_muted)
                };
                let label_color = if active {
                    theme.secondary
                } else if picked {
                    theme.success
                } else {
                    theme.text
                };
                let bg = if active { theme.background_element } else { theme.background_panel };

                let prefix = if q.multiple {
                    format!("  [{}] {}.", if picked { "x" } else { " " }, i + 1)
                } else {
                    format!("  {}.", i + 1)
                };
                let label_text = if q.multiple {
                    format!(" {}", opt.label)
                } else {
                    format!(" {}", opt.label)
                };
                let check = if !q.multiple && picked { " *" } else { "" };

                lines.push(Line::from(vec![
                    Span::styled(prefix, num_style.bg(bg)),
                    Span::styled(label_text, Style::default().fg(label_color).bg(bg)),
                    Span::styled(check.to_string(), Style::default().fg(theme.success).bg(bg)),
                ]));

                if let Some(ref desc) = opt.description {
                    lines.push(Line::from(Span::styled(
                        format!("     {}", desc),
                        Style::default().fg(theme.text_muted),
                    )));
                }
            }

            // Custom option
            if q.custom {
                let idx = q.options.len();
                let active = self.selected == idx;
                let picked = self.custom_picked();
                let bg = if active { theme.background_element } else { theme.background_panel };
                let label_color = if active {
                    theme.secondary
                } else if picked {
                    theme.success
                } else {
                    theme.text
                };
                let prefix = if q.multiple {
                    format!("  [{}] {}.", if picked { "x" } else { " " }, idx + 1)
                } else {
                    format!("  {}.", idx + 1)
                };

                if self.editing {
                    let text = self.custom_text.get(self.tab).cloned().unwrap_or_default();
                    lines.push(Line::from(vec![
                        Span::styled(prefix, Style::default().fg(theme.text_muted).bg(bg)),
                        Span::styled(" Type your own answer", Style::default().fg(label_color).bg(bg)),
                    ]));
                    lines.push(Line::from(Span::styled(
                        format!("     > {}", text),
                        Style::default().fg(theme.text),
                    )));
                } else {
                    let text = self.custom_text.get(self.tab).cloned().unwrap_or_default();
                    if !text.is_empty() {
                        lines.push(Line::from(vec![
                            Span::styled(prefix, Style::default().fg(theme.text_muted).bg(bg)),
                            Span::styled(" Type your own answer", Style::default().fg(label_color).bg(bg)),
                            Span::styled(" *", Style::default().fg(theme.success).bg(bg)),
                        ]));
                        lines.push(Line::from(Span::styled(
                            format!("     {}", text),
                            Style::default().fg(theme.text_muted),
                        )));
                    } else {
                        lines.push(Line::from(vec![
                            Span::styled(prefix, Style::default().fg(theme.text_muted).bg(bg)),
                            Span::styled(" Type your own answer", Style::default().fg(label_color).bg(bg)),
                        ]));
                    }
                }
            }
        }

        // Hints
        lines.push(Line::from(""));
        let mut hint_spans: Vec<Span> = Vec::new();
        if !self.single() {
            hint_spans.push(Span::styled("<->", Style::default().fg(theme.text)));
            hint_spans.push(Span::raw(" "));
            hint_spans.push(Span::styled("tab", Style::default().fg(theme.text_muted)));
            hint_spans.push(Span::raw("  "));
        }
        if !self.is_confirm_tab() {
            hint_spans.push(Span::styled("up/down", Style::default().fg(theme.text)));
            hint_spans.push(Span::raw(" "));
            hint_spans.push(Span::styled("select", Style::default().fg(theme.text_muted)));
            hint_spans.push(Span::raw("  "));
        }
        hint_spans.push(Span::styled("enter", Style::default().fg(theme.text)));
        hint_spans.push(Span::raw(" "));
        let enter_hint = if self.is_confirm_tab() {
            "submit"
        } else if self.single() {
            "submit"
        } else {
            let q = self.current_question();
            if q.map(|q| q.multiple).unwrap_or(false) {
                "toggle"
            } else {
                "confirm"
            }
        };
        hint_spans.push(Span::styled(enter_hint, Style::default().fg(theme.text_muted)));
        hint_spans.push(Span::raw("  "));
        hint_spans.push(Span::styled("esc", Style::default().fg(theme.text)));
        hint_spans.push(Span::raw(" "));
        hint_spans.push(Span::styled("dismiss", Style::default().fg(theme.text_muted)));
        lines.push(Line::from(hint_spans));

        let block = Block::default()
            .borders(Borders::LEFT)
            .border_style(Style::default().fg(theme.accent))
            .style(Style::default().bg(theme.background_panel));

        f.render_widget(block, area);
        f.render_widget(
            Paragraph::new(lines).style(Style::default().bg(theme.background_panel)),
            Rect {
                x: area.x + 1,
                y: area.y,
                width: area.width.saturating_sub(1),
                height: area.height,
            },
        );

        // Cursor for editing mode
        if self.editing {
            let text_len = self.custom_text.get(self.tab).map(|t| t.len()).unwrap_or(0) as u16;
            f.set_cursor_position((area.x + 7 + text_len, area.y + 5));
        }
    }
}
