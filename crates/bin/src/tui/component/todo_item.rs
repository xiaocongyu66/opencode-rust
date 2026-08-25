//! Todo item component — renders a single todo with status indicator.
//! Ported from tui/src/component/todo-item.tsx

use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::layout::Rect;
use ratatui::widgets::Paragraph;
use ratatui::Frame;
use crate::tui::theme::Theme;

pub struct TodoItem {
    pub status: TodoStatus,
    pub content: String,
    pub priority: String,
}

#[derive(Clone, PartialEq)]
pub enum TodoStatus {
    Pending,
    InProgress,
    Completed,
    Cancelled,
}

impl TodoStatus {
    pub fn from_str(s: &str) -> Self {
        match s {
            "in_progress" => TodoStatus::InProgress,
            "completed" => TodoStatus::Completed,
            "cancelled" => TodoStatus::Cancelled,
            _ => TodoStatus::Pending,
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            TodoStatus::Completed => "[x]",
            TodoStatus::InProgress => "[~]",
            TodoStatus::Cancelled => "[-]",
            TodoStatus::Pending => "[ ]",
        }
    }
}

impl TodoItem {
    pub fn new(status: &str, content: &str, priority: &str) -> Self {
        Self {
            status: TodoStatus::from_str(status),
            content: content.to_string(),
            priority: priority.to_string(),
        }
    }

    pub fn render(&self, f: &mut Frame, area: Rect, theme: &Theme) {
        let (icon, color) = match self.status {
            TodoStatus::Completed => ("[x]", theme.text_muted),
            TodoStatus::InProgress => ("[~]", theme.warning),
            TodoStatus::Cancelled => ("[-]", theme.text_muted),
            TodoStatus::Pending => ("[ ]", theme.text_muted),
        };

        let priority_icon = match self.priority.as_str() {
            "high" => "!",
            "medium" => "~",
            _ => " ",
        };

        let line = Line::from(vec![
            Span::styled(format!(" {} ", icon), Style::default().fg(color)),
            Span::styled(priority_icon.to_string(), Style::default().fg(theme.error)),
            Span::raw(" "),
            Span::styled(
                self.content.clone(),
                Style::default().fg(color),
            ),
        ]);

        f.render_widget(Paragraph::new(line), area);
    }
}
