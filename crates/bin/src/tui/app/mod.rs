//! TUI application — main loop, state management, event handling, version check, startup loading.
//! Ported from tui/src/app.tsx (1134 lines) + tui/src/index.tsx + tui/src/keymap.tsx (290 lines)
//!
//! Features:
//! - Complete application main loop with crossterm event polling
//! - Version comparison (semver with prerelease support)
//! - Startup loading screen
//! - Terminal title management (route-based)
//! - Command palette with slash commands
//! - Plugin route rendering
//! - Attention (notification/sound) integration
//! - External editor integration ($EDITOR/$VISUAL, Zed)
//! - Clipboard copy via OSC 52
//! - Signal handling (SIGHUP, SIGTSTP/SIGCONT)
//! - Keymap with leader keys, comma bindings, mode stack

pub mod app_impl;
pub mod click_registry;
pub mod key_handler;
pub mod message;
pub mod util;

pub use key_handler::*;
pub use message::{input_preview, refresh_message_text, ChatMessage, ChatPart, MessageRole, ToolPartState};
pub use util::{dbg_log, error_message, is_version_greater, parse_version};

use std::env;
use std::io::{self, Write};
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::execute;
use ratatui::{backend::CrosstermBackend, Terminal};
use tokio::sync::mpsc;

use crate::tui::attention::{AttentionConfig, TuiAttention};
use crate::tui::component::command_palette::CommandPalette;
use crate::tui::component::dialog::{Dialog, DialogResult};
use crate::tui::component::footer::Footer;
use crate::tui::component::prompt::{Prompt, PromptAction};
use crate::tui::component::sidebar::Sidebar;
use crate::tui::component::spinner::Spinner;
use crate::tui::component::toast::ToastManager;
use crate::tui::editor;
use crate::tui::event::InputMode;
use crate::tui::runtime::{self, RuntimeState, TerminalEnvironment, TuiStartup};
use crate::tui::theme::{self, Theme};
use crate::tui::routes::permission::PermissionState;
use crate::tui::routes::question::QuestionState;
use crate::tui::ui::render;

use crate::core::session::runner::{RunnerEvent, SessionRunner};
use crate::core::session::store::InMemorySessionStore;
use crate::core::session::SessionStore;
use crate::llm::provider_factory;
use crate::schema::ids::SessionID;
use crate::schema::session::{SessionInfo, SessionMessage, SessionTime, AssistantContent, AssistantToolTime, AssistantTime};
use crate::schema::model::ModelRef;
use crate::schema::ids::{SessionMessageID, ModelID, ProviderID, AgentID};
use crate::schema::location::LocationRef;
use crate::schema::common::AbsolutePath;
use crate::schema::ids::ProjectID;

// ---------------------------------------------------------------------------
// Constants — binding commands from app.tsx
// ---------------------------------------------------------------------------

const APP_GLOBAL_BINDING_COMMANDS: &[&str] = &[
    "session.list",
    "session.new",
    "session.quick_switch.1",
    "session.quick_switch.2",
    "session.quick_switch.3",
    "session.quick_switch.4",
    "session.quick_switch.5",
    "session.quick_switch.6",
    "session.quick_switch.7",
    "session.quick_switch.8",
    "session.quick_switch.9",
];

const APP_BINDING_COMMANDS: &[&str] = &[
    "command.palette.show",
    "model.list",
    "model.cycle_recent",
    "model.cycle_recent_reverse",
    "model.cycle_favorite",
    "model.cycle_favorite_reverse",
    "agent.list",
    "mcp.list",
    "agent.cycle",
    "agent.cycle.reverse",
    "variant.cycle",
    "variant.list",
    "provider.connect",
    "console.org.switch",
    "opencode.status",
    "opencode.debug",
    "theme.switch",
    "theme.switch_mode",
    "theme.mode.lock",
    "help.show",
    "docs.open",
    "diff.open",
    "workspace.list",
    "app.debug",
    "app.console",
    "app.heap_snapshot",
    "terminal.suspend",
    "terminal.title.toggle",
    "app.toggle.animations",
    "app.toggle.file_context",
    "app.toggle.diffwrap",
    "app.toggle.paste_summary",
    "app.toggle.session_directory_filter",
];

const LEADER_TOKEN: &str = "leader";
const OPENCODE_BASE_MODE: &str = "base";
const COMMAND_PALETTE_COMMAND: &str = "command.palette.show";

// ---------------------------------------------------------------------------
// ---------------------------------------------------------------------------
// Route types
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub enum Route {
    Home,
    Session { session_id: String },
    Plugin { id: String, data: Option<String> },
}

impl Default for Route {
    fn default() -> Self {
        Route::Home
    }
}

// ---------------------------------------------------------------------------
// TUI input — mirrors `TuiInput` from app.tsx
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct TuiInput {
    pub url: String,
    pub directory: Option<String>,
    pub fetch: Option<String>,
    pub headers: Option<serde_json::Value>,
}

impl Default for TuiInput {
    fn default() -> Self {
        Self {
            url: String::new(),
            directory: None,
            fetch: None,
            headers: None,
        }
    }
}

// ---------------------------------------------------------------------------
// Exit state
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default)]
pub struct ExitState {
    pub epilogue: Option<String>,
    pub reason: Option<String>,
}


// ---------------------------------------------------------------------------
// App — main application state
// ---------------------------------------------------------------------------

pub struct App {
    pub messages: Vec<ChatMessage>,
    pub prompt: Prompt,
    pub mode: InputMode,
    pub should_quit: bool,
    pub messages_scroll: crate::tui::component::scroll_view::ScrollView,
    pub spinner: Spinner,
    pub spinner_active: bool,
    /// Clickable regions for mouse interaction. Cleared at the start of each
    /// render frame; components register their clickable areas during render.
    pub click_registry: click_registry::ClickRegistry,
    pub footer: Footer,
    pub sidebar: Sidebar,
    pub toast_manager: ToastManager,
    pub command_palette: CommandPalette,
    pub dialog: Option<Dialog>,
    pub theme: Theme,
    pub theme_mode: theme::ThemeMode,
    pub is_thinking: bool,
    pub show_sidebar: bool,
    pub permission_state: Option<PermissionState>,
    pub question_state: Option<QuestionState>,
    pub route: Route,
    pub exit_state: ExitState,
    pub runtime_state: RuntimeState,
    pub terminal_env: TerminalEnvironment,
    pub startup: TuiStartup,
    pub attention: TuiAttention,
    pub terminal_title_enabled: bool,
    pub paste_summary_enabled: bool,
    pub animations_enabled: bool,
    pub file_context_enabled: bool,
    pub session_directory_filter_enabled: bool,
    pub diff_wrap_mode: String,
    pub permission_mode_auto: bool,
    pub ready: bool,
    pub plugin_ready: bool,
    pub session_id: Option<SessionID>,
    pub runner_rx: Option<mpsc::Receiver<RunnerEvent>>,
    /// ACP event stream (post-bridge). When set, poll this instead of
    /// runner_rx — events come already converted to AcpEvent. The bridge
    /// task drains runner_rx in the background (claude-code-book Ch02/Ch13).
    pub acp_rx: Option<mpsc::Receiver<crate::core::acp::AcpEvent>>,
    pub current_assistant_text: String,
    /// Accumulated reasoning (thinking) text for the current assistant turn.
    /// Flushed to the message as a single block-quote when reasoning ends.
    pub current_reasoning_text: String,
    pub system_prompt: String,
    pub store: Arc<dyn SessionStore>,
    /// Current model display name (e.g. "claude-sonnet-4-6"). Shown in the
    /// assistant message footer next to the agent name.
    pub current_model: String,
    /// Cumulative input tokens used in the current session (for sidebar).
    pub total_input_tokens: u64,
    /// Cumulative output tokens used in the current session (for sidebar).
    pub total_output_tokens: u64,
    /// Number of tool calls made in the current session.
    pub tool_call_count: u64,
    /// Number of LLM steps (turns) in the current session.
    pub step_count: u64,
    /// Queue of messages waiting to be sent. When the LLM is busy, new
    /// submissions go here instead of being rejected; they're flushed
    /// automatically when the current turn finishes (RunnerEvent::Done).
    pub message_queue: Vec<String>,
}

// ---------------------------------------------------------------------------
// Main entry point — mirrors `run` from app.tsx
// ---------------------------------------------------------------------------

pub async fn run(resume: Option<String>) -> Result<()> {
    crate::tui::i18n::init();

    if !crossterm::tty::IsTty::is_tty(&io::stdin()) {
        println!("rsopencode — interactive terminal required");
        println!();
        println!("Usage: rsopencode              # Interactive chat (needs terminal)");
        println!("       rsopencode serve         # Start HTTP API server");
        println!("       rsopencode agents         # List agents");
        println!("       rsopencode models        # List models");
        println!("       rsopencode --resume <id>  # Resume a previous session");
        println!("       rsopencode --help        # Show all commands");
        return Ok(());
    }

    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen, crossterm::event::DisableMouseCapture);
        original_hook(info);
    }));

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, crossterm::event::EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let mut app = App::new();
    app.update_terminal_title();

    // Resume a previous session if --resume <id> was given.
    if let Some(sid) = resume.as_ref() {
        app.resume_session(sid);
    }

    // Startup loading: if not skipping, show loading screen
    let show_loading = !app.startup.skip_initial_loading;
    if show_loading {
        app.ready = false;
    } else {
        app.ready = true;
    }

    let mut last_spinner_tick = Instant::now();
    let result = main_loop(&mut terminal, &mut app, &mut last_spinner_tick).await;

    runtime::win32_flush_input_buffer();

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, crossterm::event::DisableMouseCapture)?;
    terminal.show_cursor()?;

    // Print the exit epilogue: logo + resume hint for the last session.
    print_exit_epilogue(&app);

    if let Some(reason) = &app.exit_state.reason {
        // Only print if it's not our internal __resume__ marker.
        if !reason.starts_with("__resume__:") {
            eprintln!("{}", reason);
        }
    }
    if let Some(epilogue) = &app.exit_state.epilogue {
        println!("{}", epilogue);
    }

    result
}

/// Print the exit epilogue: a small logo and a `rsopencode --resume <id>`
/// hint so the user can easily get back to their last session.
fn print_exit_epilogue(app: &App) {
    use std::io::Write;
    let mut out = io::stdout();
    let _ = writeln!(out);
    // Full opencode logo (left + right halves).
    let _ = writeln!(out, "  ▄                                                ");
    let _ = writeln!(out, "  █▀▀█ █▀▀█ █▀▀█ █▀▀▄ █▀▀▀ █▀▀█ █▀▀█ █▀▀█");
    let _ = writeln!(out, "  █  █ █  █ █▀▀▀ █  █ █    █  █ █  █ █▀▀▀");
    let _ = writeln!(out, "  ▀▀▀▀ █▀▀▀ ▀▀▀▀ ▀▀▀▀ ▀▀▀▀ ▀▀▀▀ ▀▀▀▀ ▀▀▀▀");
    let _ = writeln!(out);

    if let Some(reason) = &app.exit_state.reason {
        if let Some(sid) = reason.strip_prefix("__resume__:") {
            let _ = writeln!(out, "  Resume this session with:");
            let _ = writeln!(out, "    rsopencode --resume {}", sid);
            let _ = writeln!(out);
        }
    }
    let _ = out.flush();
}

pub fn handle_update_available(
    app: &mut App,
    version: &str,
    skipped_version: Option<&str>,
) -> bool {
    if let Some(skipped) = skipped_version {
        if !is_version_greater(version, skipped) {
            return false;
        }
    }
    app.toast_manager.show(
        crate::t!("tui.toast.update_available", version = version).to_string(),
        crate::tui::component::toast::ToastVariant::Info,
    );
    true
}

// ---------------------------------------------------------------------------
// Session event handlers — mirrors event.on("session.*") handlers
// ---------------------------------------------------------------------------

pub fn handle_session_deleted(app: &mut App, session_id: &str) {
    if let Route::Session { session_id: current } = &app.route {
        if current == session_id {
            app.route = Route::Home;
            app.toast_manager.show(
                crate::t!("tui.session.deleted_current"),
                crate::tui::component::toast::ToastVariant::Info,
            );
        }
    }
}

pub fn handle_session_error(app: &mut App, error: &serde_json::Value) {
    if let Some(name) = error.get("name").and_then(|v| v.as_str()) {
        if name == "MessageAbortedError" {
            return;
        }
    }
    let message = error_message(error);
    app.toast_manager.show(
        &message,
        crate::tui::component::toast::ToastVariant::Error,
    );
}

pub fn handle_session_select(app: &mut App, session_id: &str) {
    app.set_route(Route::Session {
        session_id: session_id.to_string(),
    });
}

// ---------------------------------------------------------------------------
// Clipboard handler — mirrors renderer.console.onCopySelection
// ---------------------------------------------------------------------------

pub fn handle_copy_selection(app: &mut App, text: &str) {
    if text.is_empty() {
        return;
    }
    runtime::clipboard_write(text);
    app.toast_manager.show(
        crate::t!("tui.toast.copied"),
        crate::tui::component::toast::ToastVariant::Info,
    );
}

// ---------------------------------------------------------------------------
// TUI export — mirrors index.tsx
// ---------------------------------------------------------------------------

pub use run as tui_run;
pub use App as TuiApp;
pub use TuiInput as TuiInputType;


#[cfg(test)]
mod tests;
