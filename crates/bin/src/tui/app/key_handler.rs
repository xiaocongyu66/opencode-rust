use super::*;
// ---------------------------------------------------------------------------
// Main loop — event polling, rendering, state updates
// ---------------------------------------------------------------------------

pub(super) async fn main_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    last_spinner_tick: &mut Instant,
) -> Result<()> {
    let loading_start = Instant::now();

    loop {
        // Clear the click registry before each render so components can
        // re-register their clickable regions with fresh coordinates.
        app.click_registry.clear();
        terminal.draw(|f| render(f, app))?;

        // Startup loading: transition to ready after a brief delay
        if !app.ready {
            if loading_start.elapsed() >= Duration::from_millis(500) {
                app.ready = true;
            }
        }

        // Tick spinner and toast — always tick, even without events,
        // so the spinner animates smoothly.
        if app.spinner_active && last_spinner_tick.elapsed() >= Duration::from_millis(80) {
            app.spinner.tick();
            *last_spinner_tick = Instant::now();
        }
        app.toast_manager.tick();

        // Poll runner events
        app.poll_runner_events();

        // Poll events — use 16ms (~60fps) for low-latency mouse/keyboard.
        // The original opentui is event-driven (no polling), but crossterm
        // requires poll. 16ms gives near-instant response for mouse scroll
        // and clicks.
        if event::poll(Duration::from_millis(16))? {
            // Process ALL pending events (not just one) to avoid lag.
            loop {
                match event::read()? {
                    Event::Key(key) => {
                        if key.kind != KeyEventKind::Press {
                            continue;
                        }
                        handle_key_event(terminal, app, key);
                    }
                    Event::Mouse(mouse) => {
                        handle_mouse_event(app, mouse);
                    }
                    Event::Resize(_, _) => {
                        // Will redraw next loop
                    }
                    _ => {}
                }
                // Check if more events are pending — drain them all.
                if !event::poll(Duration::from_millis(0))? {
                    break;
                }
            }
        }

        if app.should_quit {
            break;
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Key event handler — priority: Ctrl+C > Command Palette > Dialog > Permission > Question > Mode
// ---------------------------------------------------------------------------

pub(super) fn handle_key_event(
    _terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    key: event::KeyEvent,
) {
    if key.modifiers.contains(KeyModifiers::CONTROL) && (key.code == KeyCode::Char('c') || key.code == KeyCode::Char('d')) {
        app.should_quit = true;
        return;
    }

    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('p') {
        app.command_palette.toggle();
        return;
    }

    // Ctrl-j / Ctrl-k: jump to next / previous message boundary.
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('j') {
        app.messages_scroll.scroll_to_next_message(true);
        return;
    }
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('k') {
        app.messages_scroll.scroll_to_next_message(false);
        return;
    }

    if app.command_palette.visible {
        if let Some(action) = app.command_palette.handle_key(key) {
            handle_command_action(app, &action);
        }
        return;
    }

    if let Some(ref mut dialog) = app.dialog {
        let dialog_title = dialog.title.clone();
        match dialog.handle_key(key) {
            DialogResult::Select(val) => {
                app.dialog = None;
                // If this was the model picker, switch the model.
                if dialog_title == "Select Model" {
                    // `val` is the DialogOption value, which we set to the model id.
                    let model_id = val.clone();
                    use crate::llm::provider_factory;
                    if let Some(name) = provider_factory::switch_model(&model_id) {
                        app.set_model(name.clone(), String::new());
                        // Update config.toml.
                        if let Some(home) = dirs::home_dir() {
                            let path = home.join(".rsopencode").join("config.toml");
                            if let Ok(content) = std::fs::read_to_string(&path) {
                                let updated = crate::tui::app::app_impl::update_config_default_model(&content, &model_id);
                                let _ = std::fs::write(&path, updated);
                            }
                        }
                        app.toast_manager.show(
                            format!("Switched to: {}", name),
                            crate::tui::component::toast::ToastVariant::Success,
                        );
                    } else {
                        app.toast_manager.show(
                            format!("Could not switch to {}", val),
                            crate::tui::component::toast::ToastVariant::Error,
                        );
                    }
                } else {
                    app.toast_manager.show(
                        crate::t!("tui.toast.selected", value = val.as_str()).to_string(),
                        crate::tui::component::toast::ToastVariant::Info,
                    );
                }
            }
            DialogResult::Confirm => {
                app.dialog = None;
            }
            DialogResult::Cancel => {
                app.dialog = None;
            }
            DialogResult::Close => {
                app.dialog = None;
            }
            DialogResult::None => {}
        }
        return;
    }

    if let Some(ref mut perm) = app.permission_state {
        use crate::tui::routes::permission::{PermissionResult, PermissionReply};
        match perm.handle_key(key) {
            PermissionResult::Reply(PermissionReply::Once, _) => {
                app.permission_state = None;
                app.toast_manager.show(
                    crate::t!("tui.permission.allowed_once"),
                    crate::tui::component::toast::ToastVariant::Success,
                );
            }
            PermissionResult::Reply(PermissionReply::Always, _) => {
                app.permission_state = None;
                app.toast_manager.show(
                    crate::t!("tui.permission.allowed_always"),
                    crate::tui::component::toast::ToastVariant::Success,
                );
            }
            PermissionResult::Reply(PermissionReply::Reject, _) => {
                app.permission_state = None;
                app.toast_manager.show(
                    crate::t!("tui.permission.rejected"),
                    crate::tui::component::toast::ToastVariant::Warning,
                );
            }
            PermissionResult::None => {}
        }
        return;
    }

    if let Some(ref mut qs) = app.question_state {
        use crate::tui::routes::question::QuestionResult;
        match qs.handle_key(key) {
            QuestionResult::Submit(_) => {
                app.question_state = None;
                app.toast_manager.show(
                    crate::t!("tui.question.submitted"),
                    crate::tui::component::toast::ToastVariant::Success,
                );
            }
            QuestionResult::Reject => {
                app.question_state = None;
                app.toast_manager.show(
                    crate::t!("tui.question.dismissed"),
                    crate::tui::component::toast::ToastVariant::Warning,
                );
            }
            QuestionResult::None => {}
        }
        return;
    }

    match app.mode {
        InputMode::Normal => handle_normal_key(app, key),
        InputMode::Insert => handle_insert_key(app, key),
        InputMode::Help => handle_help_key(app, key),
    }
}

pub(super) fn handle_normal_key(app: &mut App, key: event::KeyEvent) {
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => app.exit(),
        KeyCode::Char('h') | KeyCode::Char('?') => {
            app.mode = InputMode::Help;
        }
        KeyCode::Char(':') => {
            app.command_palette.toggle();
        }
        KeyCode::Char('s') => {
            app.show_sidebar = !app.show_sidebar;
        }
        KeyCode::Char('e') => {
            app.open_external_editor();
        }
        KeyCode::Char('t') => {
            app.toggle_terminal_title();
            let msg = if app.terminal_title_enabled { crate::t!("tui.toast.terminal_title_enabled") } else { crate::t!("tui.toast.terminal_title_disabled") };
            app.toast_manager.show(
                msg.to_string(),
                crate::tui::component::toast::ToastVariant::Info,
            );
        }
        KeyCode::Char('p') => {
            app.toggle_permission_mode();
            let msg = if app.permission_mode_auto { crate::t!("tui.toast.auto_approve_enabled") } else { crate::t!("tui.toast.auto_approve_disabled") };
            app.toast_manager.show(
                msg.to_string(),
                crate::tui::component::toast::ToastVariant::Info,
            );
        }
        KeyCode::Up => app.messages_scroll.on_line_delta(-1),
        KeyCode::Down => app.messages_scroll.on_line_delta(1),
        KeyCode::PageUp => app.messages_scroll.on_line_delta(-10),
        KeyCode::PageDown => app.messages_scroll.on_line_delta(10),
        // Any other printable char → go back to insert mode and inject the char
        KeyCode::Char(c) => {
            app.mode = InputMode::Insert;
            app.prompt.focused = true;
            app.footer.status = crate::t!("tui.status.insert").to_string();
            app.prompt.handle_key(event::KeyEvent::new(KeyCode::Char(c), key.modifiers));
        }
        _ => {}
    }
}

pub(super) fn handle_insert_key(app: &mut App, key: event::KeyEvent) {
    let action = app.prompt.handle_key(key);
    match action {
        PromptAction::Submit(text) => {
            app.submit_message(text);
            app.prompt.focused = true;
        }
        PromptAction::Cancel => {
            // If the LLM is generating, Esc interrupts it (drops the runner
            // channel so the background task's sends fail and it exits).
            if app.is_thinking {
                app.runner_rx = None;
                app.is_thinking = false;
                app.spinner_active = false;
                app.footer.status = crate::t!("tui.status.idle").to_string();
                app.toast_manager.show(
                    crate::t!("tui.error.interrupted").to_string(),
                    crate::tui::component::toast::ToastVariant::Warning,
                );
            }
            app.mode = InputMode::Normal;
            app.prompt.focused = false;
        }
        _ => {}
    }
}

pub(super) fn handle_help_key(app: &mut App, key: event::KeyEvent) {
    match key.code {
        KeyCode::Esc | KeyCode::Enter | KeyCode::Char('q') => {
            app.mode = InputMode::Normal;
        }
        _ => {}
    }
}

// ---------------------------------------------------------------------------
// Command action handler — mirrors app commands from app.tsx
// ---------------------------------------------------------------------------

pub(super) fn handle_command_action(app: &mut App, action: &str) {
    match action {
        "quit" | "app.exit" => app.exit(),
        "help" | "help.show" => app.mode = InputMode::Help,
        "switch_agent" | "agent.list" => {
            app.dialog = Some(crate::tui::component::dialog::Dialog::select(
                crate::t!("tui.agent.switch").to_string(),
                vec![
                    crate::tui::component::dialog::DialogOption {
                        title: "build".to_string(),
                        description: Some(crate::t!("tui.agent.build_desc").to_string()),
                        value: "build".to_string(),
                        details: vec![],
                        category: None,
                        disabled: false,
                    },
                    crate::tui::component::dialog::DialogOption {
                        title: "plan".to_string(),
                        description: Some(crate::t!("tui.agent.plan_desc").to_string()),
                        value: "plan".to_string(),
                        details: vec![],
                        category: None,
                        disabled: false,
                    },
                ],
            ));
        }
        "switch_theme" | "theme.switch" => {
            app.dialog = Some(crate::tui::component::dialog::Dialog::select(
                crate::t!("tui.theme.switch").to_string(),
                crate::tui::theme::THEME_NAMES
                    .iter()
                    .map(|name| crate::tui::component::dialog::DialogOption {
                        title: name.to_string(),
                        description: None,
                        value: name.to_string(),
                        details: vec![],
                        category: None,
                        disabled: false,
                    })
                    .collect(),
            ));
        }
        "theme.switch_mode" => {
            let new_theme = if app.theme_mode == theme::ThemeMode::Dark {
                theme::ThemeMode::Light
            } else {
                theme::ThemeMode::Dark
            };
            app.theme_mode = new_theme;
            app.dialog = None;
        }
        "new_session" | "session.new" => {
            app.messages.clear();
            app.messages.push(ChatMessage::new(
                MessageRole::System,
                crate::t!("tui.session.new_started").to_string(),
            ));
            app.route = Route::Home;
            app.toast_manager.show(
                crate::t!("tui.toast.new_session"),
                crate::tui::component::toast::ToastVariant::Success,
            );
        }
        "switch_model" | "model.list" => {
            app.toast_manager.show(
                crate::t!("tui.toast.model_switch"),
                crate::tui::component::toast::ToastVariant::Warning,
            );
        }
        "list_sessions" | "session.list" => {
            app.toast_manager.show(
                crate::t!("tui.toast.no_history"),
                crate::tui::component::toast::ToastVariant::Info,
            );
        }
        "opencode.status" => {
            app.toast_manager.show(
                &format!(
                    "Platform: {} | Multiplexer: {} | Display: {}",
                    app.terminal_env.platform,
                    app.terminal_env.multiplexer.as_deref().unwrap_or("none"),
                    app.terminal_env.display_server.as_deref().unwrap_or("none"),
                ),
                crate::tui::component::toast::ToastVariant::Info,
            );
        }
        "terminal.title.toggle" => {
            app.toggle_terminal_title();
            app.dialog = None;
        }
        "app.toggle.animations" => {
            app.toggle_animations();
            app.dialog = None;
        }
        "app.toggle.file_context" => {
            app.toggle_file_context();
            app.dialog = None;
        }
        "app.toggle.diffwrap" => {
            app.toggle_diff_wrap();
            app.dialog = None;
        }
        "app.toggle.paste_summary" => {
            app.toggle_paste_summary();
            app.dialog = None;
        }
        "app.toggle.session_directory_filter" => {
            app.toggle_session_directory_filter();
            app.dialog = None;
        }
        "permission.mode" => {
            app.toggle_permission_mode();
            app.dialog = None;
        }
        "session.toggle.scrollbar" => {
            app.toggle_scrollbar();
            app.dialog = None;
        }
        "terminal.suspend" => {
            app.suspend_terminal();
        }
        "docs.open" => {
            app.toast_manager.show(
                crate::t!("tui.toast.docs_open"),
                crate::tui::component::toast::ToastVariant::Info,
            );
            app.dialog = None;
        }
        "app.console" => {
            app.toast_manager.show(
                crate::t!("tui.toast.console"),
                crate::tui::component::toast::ToastVariant::Info,
            );
            app.dialog = None;
        }
        "app.debug" => {
            app.toast_manager.show(
                crate::t!("tui.toast.debug"),
                crate::tui::component::toast::ToastVariant::Info,
            );
            app.dialog = None;
        }
        _ => {}
    }
}

// ---------------------------------------------------------------------------
// Update notification handler — mirrors `installation.update-available` event
// ---------------------------------------------------------------------------


// ---------------------------------------------------------------------------
// Mouse event handler
// ---------------------------------------------------------------------------

/// Handle a crossterm mouse event. Looks up the click in the registry and
/// dispatches the corresponding action.
pub(super) fn handle_mouse_event(app: &mut App, mouse: event::MouseEvent) {
    use event::MouseEventKind;
    crate::tui::app::util::dbg_log(&format!("mouse event: {:?} at ({},{})", mouse.kind, mouse.column, mouse.row));

    match mouse.kind {
        MouseEventKind::Moved => {
            // Update hover state; if changed, the next render will reflect it.
            app.click_registry.update_hover(mouse.column, mouse.row);
        }
        MouseEventKind::Down(event::MouseButton::Left) => {
            // Detect whether the press landed on the scrollbar track. If so,
            // subsequent Drag events scrub to a position instead of panning.
            let on_bar = app
                .messages_scroll
                .scrollbar_area
                .map(|r| {
                    mouse.column >= r.x
                        && mouse.column < r.x + r.width
                        && mouse.row >= r.y
                        && mouse.row < r.y + r.height
                })
                .unwrap_or(false);
            if !on_bar {
                if let Some(region) = app.click_registry.hit_test(mouse.column, mouse.row) {
                    let action = region.action.0.clone();
                    dispatch_click_action(app, &action);
                }
            }
            app.messages_scroll.on_drag_start(mouse.row, on_bar);
        }
        MouseEventKind::ScrollUp => app.messages_scroll.on_wheel_up(),
        MouseEventKind::ScrollDown => app.messages_scroll.on_wheel_down(),
        MouseEventKind::Drag(event::MouseButton::Left) => {
            app.messages_scroll.on_drag(mouse.row);
        }
        MouseEventKind::Up(event::MouseButton::Left) => {
            app.messages_scroll.on_drag_end();
        }
        _ => {}
    }
}

/// Dispatch a click action string to the corresponding App behavior.
/// Action format: `namespace:detail` (e.g. `model:select`, `session:switch:ses_xxx`).
fn dispatch_click_action(app: &mut App, action: &str) {
    let mut parts = action.splitn(2, ':');
    let namespace = parts.next().unwrap_or("");
    let detail = parts.next().unwrap_or("");

    match namespace {
        "model" => {
            // Open the model selection dialog (same as /model command).
            use crate::llm::provider_factory;
            use crate::tui::component::dialog::{Dialog, DialogOption};
            let models = provider_factory::list_configured_models();
            if models.is_empty() {
                app.toast_manager.show(
                    "No models configured. Edit ~/.rsopencode/config.toml".to_string(),
                    crate::tui::component::toast::ToastVariant::Warning,
                );
            } else {
                let options: Vec<DialogOption> = models.iter().map(|(provider_id, id, name)| {
                    let title = if id == name { id.clone() } else { format!("{} ({})", name, id) };
                    DialogOption::new(title, id.clone())
                        .with_description(format!("provider: {}", provider_id))
                }).collect();
                app.dialog = Some(Dialog::select("Select Model", options));
            }
        }
        "session" => {
            // session:switch:<id>
            let id = detail.strip_prefix("switch:").unwrap_or("");
            if !id.is_empty() {
                app.resume_session(id);
            }
        }
        "command" => {
            // command:run:<name>
            let cmd = detail.strip_prefix("run:").unwrap_or("");
            if !cmd.is_empty() {
                app.handle_slash_command(cmd, String::new());
            }
        }
        "scroll" => {
            if detail == "bottom" {
                app.scroll_to_bottom();
            } else if detail == "top" {
                app.messages_scroll.scroll_to_top();
            }
        }
        "theme" => {
            // theme:cycle — cycle to the next theme
            let names = crate::tui::theme::loader::list_theme_names();
            let current = "opencode";
            let idx = names.iter().position(|n| *n == current).unwrap_or(0);
            let next = names[(idx + 1) % names.len()];
            app.set_theme(next);
            app.toast_manager.show(
                format!("Theme: {}", next),
                crate::tui::component::toast::ToastVariant::Info,
            );
        }
        _ => {
            // Unknown action — ignore.
        }
    }
}
