//! Message rendering — renders chat messages (user, assistant, system, tool).
//! Ported from tui/src/routes/session/index.tsx message rendering

use ratatui::style::{Style, Modifier};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::layout::Rect;
use ratatui::Frame;
use crate::tui::theme::Theme;
use crate::tui::app::ChatMessage;

pub fn render_message(f: &mut Frame, area: Rect, msg: &ChatMessage, theme: &Theme, width: u16) {
    let (prefix, prefix_color, text_color) = match msg.role {
        crate::tui::app::MessageRole::User => (
            crate::t!("tui.message.user_prefix").to_string(),
            theme.secondary,
            theme.text,
        ),
        crate::tui::app::MessageRole::Assistant => (
            crate::t!("tui.message.assistant_prefix").to_string(),
            theme.primary,
            theme.text,
        ),
        crate::tui::app::MessageRole::System => (
            crate::t!("tui.message.system_prefix").to_string(),
            theme.text_muted,
            theme.text_muted,
        ),
    };

    let mut lines: Vec<Line> = Vec::new();

    // Header line: role name
    lines.push(Line::from(vec![
        Span::styled(
            prefix.clone(),
            Style::default().fg(prefix_color).add_modifier(Modifier::BOLD),
        ),
    ]));

    // Message body — wrap text
    let text = &msg.text;
    let max_width = (width.saturating_sub(4)) as usize;

    let mut current_line = String::new();
    for word in text.split_whitespace() {
        if current_line.len() + word.len() + 1 > max_width && !current_line.is_empty() {
            lines.push(Line::from(vec![Span::styled(
                current_line.clone(),
                Style::default().fg(text_color),
            )]));
            current_line.clear();
        }
        if !current_line.is_empty() {
            current_line.push(' ');
        }
        current_line.push_str(word);
    }
    if !current_line.is_empty() {
        lines.push(Line::from(vec![Span::styled(
            current_line,
            Style::default().fg(text_color),
        )]));
    }

    // Empty line after message
    lines.push(Line::from(""));

    let para = Paragraph::new(lines);
    f.render_widget(para, area);
}
