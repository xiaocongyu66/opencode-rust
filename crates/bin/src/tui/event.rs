//! Input modes for the TUI.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    Insert,
    Help,
}

impl std::fmt::Display for InputMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InputMode::Normal => write!(f, "NORMAL"),
            InputMode::Insert => write!(f, "INSERT"),
            InputMode::Help => write!(f, "HELP"),
        }
    }
}
