#[derive(Debug, Clone)]
pub struct TuiPaths {
    pub cwd: String,
    pub home: String,
    pub state: String,
    pub worktree: String,
}

impl TuiPaths {
    pub fn new(cwd: String, home: String, state: String, worktree: String) -> Self {
        Self {
            cwd,
            home,
            state,
            worktree,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TuiTerminalEnvironment {
    pub platform: String,
    pub multiplexer: Option<Multiplexer>,
    pub display_server: Option<DisplayServer>,
}

#[derive(Debug, Clone)]
pub enum Multiplexer {
    Tmux,
    Screen,
}

#[derive(Debug, Clone)]
pub enum DisplayServer {
    Wayland,
    X11,
}

#[derive(Debug, Clone)]
pub struct TuiStartup {
    pub initial_route: Option<serde_json::Value>,
    pub skip_initial_loading: bool,
}

impl TuiStartup {
    pub fn new() -> Self {
        Self {
            initial_route: None,
            skip_initial_loading: false,
        }
    }
}

impl Default for TuiStartup {
    fn default() -> Self {
        Self::new()
    }
}
