//! TUI application — main loop and state management.

use std::io;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::execute;
use ratatui::{backend::CrosstermBackend, Terminal};
use anyhow::Result;

use crate::event::InputMode;
use crate::ui::render;

pub struct App {
    pub input: String,
    pub messages: Vec<String>,
    pub mode: InputMode,
    pub should_quit: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            input: String::new(),
            messages: Vec::new(),
            mode: InputMode::Normal,
            should_quit: false,
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

pub async fn run() -> Result<()> {
    crate::i18n::init();

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();

    let result = main_loop(&mut terminal, &mut app).await;

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

async fn main_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
) -> Result<()> {
    loop {
        terminal.draw(|f| render(f, app))?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                match app.mode {
                    InputMode::Normal => match key.code {
                        KeyCode::Char('q') => app.should_quit = true,
                        KeyCode::Char('i') => app.mode = InputMode::Insert,
                        KeyCode::Char('h') => app.mode = InputMode::Help,
                        _ => {}
                    },
                    InputMode::Insert => match key.code {
                        KeyCode::Esc => app.mode = InputMode::Normal,
                        KeyCode::Enter => {
                            if !app.input.is_empty() {
                                app.messages.push(app.input.clone());
                                app.input.clear();
                            }
                        }
                        KeyCode::Char(c) => app.input.push(c),
                        KeyCode::Backspace => {
                            app.input.pop();
                        }
                        _ => {}
                    },
                    InputMode::Help => match key.code {
                        KeyCode::Esc | KeyCode::Enter => app.mode = InputMode::Normal,
                        _ => {}
                    },
                }
            }
        }

        if app.should_quit {
            break;
        }
    }
    Ok(())
}
