//! TUI rendering — draws the terminal interface using ratatui.
use rust_i18n::t;

use ratatui::{Frame, layout::{Constraint, Direction, Layout, Rect}, style::{Color, Modifier, Style}, text::{Line, Span}, widgets::{Block, Borders, List, ListItem, Paragraph}};

use crate::app::App;
use crate::event::InputMode;

pub fn render(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(3),
            Constraint::Length(1),
        ])
        .split(f.area());

    render_header(f, chunks[0]);
    render_messages(f, chunks[1], app);
    render_input(f, chunks[2], app);
    render_status(f, chunks[3], app);
}

fn render_header(f: &mut Frame, area: Rect) {
    let title = t!("tui.title").to_string();
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" {} ", title))
        .style(Style::default().fg(Color::Cyan));
    f.render_widget(block, area);
}

fn render_messages(f: &mut Frame, area: Rect, app: &App) {
    let items: Vec<ListItem> = app
        .messages
        .iter()
        .enumerate()
        .map(|(i, msg)| {
            ListItem::new(Line::from(vec![
                Span::styled(format!("{} ", i + 1), Style::default().fg(Color::DarkGray)),
                Span::raw(msg),
            ]))
        })
        .collect();

    let block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default());

    if items.is_empty() {
        let empty = t!("tui.session.empty").to_string();
        f.render_widget(Paragraph::new(empty).block(block), area);
    } else {
        f.render_widget(List::new(items).block(block), area);
    }
}

fn render_input(f: &mut Frame, area: Rect, app: &App) {
    let (title, style) = match app.mode {
        InputMode::Normal => (
            t!("tui.common.commands").to_string(),
            Style::default().fg(Color::Yellow),
        ),
        InputMode::Insert => (
            t!("tui.prompt.placeholder").to_string(),
            Style::default().fg(Color::Green),
        ),
        InputMode::Help => (
            t!("tui.common.commands").to_string(),
            Style::default().fg(Color::Blue),
        ),
    };

    let block = Block::default().borders(Borders::ALL).title(format!(" {} ", title)).style(style);

    let paragraph = if app.mode == InputMode::Insert {
        Paragraph::new(app.input.as_str()).block(block)
    } else if app.mode == InputMode::Help {
        let help_text = "q: quit  i: insert  Esc: normal  Enter: send";
        Paragraph::new(help_text).block(block)
    } else {
        Paragraph::new("").block(block)
    };
    f.render_widget(paragraph, area);
}

fn render_status(f: &mut Frame, area: Rect, app: &App) {
    let (status_text, color) = match app.mode {
        InputMode::Normal => (t!("tui.status.idle").to_string(), Color::DarkGray),
        InputMode::Insert => (t!("tui.status.busy").to_string(), Color::Green),
        InputMode::Help => (t!("tui.common.commands").to_string(), Color::Blue),
    };

    let line = Line::from(vec![
        Span::styled("● ", Style::default().fg(color)),
        Span::styled(status_text, Style::default().add_modifier(Modifier::DIM)),
        Span::raw("  "),
        Span::styled("q:quit  i:insert  h:help", Style::default().fg(Color::DarkGray)),
    ]);

    f.render_widget(Paragraph::new(line), area);
}
