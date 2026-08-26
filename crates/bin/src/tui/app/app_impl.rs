use super::*;
impl App {
    pub fn new() -> Self {
        let theme = theme::themes::opencode();
        let attention = TuiAttention::new(AttentionConfig::default(), None);
        Self {
            messages: vec![ChatMessage::new(
                MessageRole::System,
                crate::t!("tui.welcome").to_string(),
            )],
            prompt: {
                let mut p = Prompt::new();
                // Pre-load model/provider display from config so the home
                // prompt shows the right model immediately (not "no model").
                if let Some(sel) = provider_factory::select_from_config() {
                    p.model = sel.model_name.clone();
                    p.provider = sel.provider_name.clone();
                }
                p
            },
            mode: InputMode::Insert,
            should_quit: false,
            messages_scroll: crate::tui::component::scroll_view::ScrollView::new(),
            spinner: Spinner::new(crate::t!("tui.status.thinking").to_string()),
            spinner_active: false,
            click_registry: crate::tui::app::click_registry::ClickRegistry::new(),
            footer: Footer::new(),
            sidebar: Sidebar::new(),
            toast_manager: ToastManager::new(),
            command_palette: CommandPalette::new(),
            dialog: None,
            theme,
            theme_mode: theme::ThemeMode::Dark,
            is_thinking: false,
            show_sidebar: true,
            permission_state: None,
            question_state: None,
            route: Route::Home,
            exit_state: ExitState::default(),
            runtime_state: RuntimeState::new(),
            terminal_env: TerminalEnvironment::detect(),
            startup: TuiStartup::from_env(),
            attention,
            terminal_title_enabled: true,
            paste_summary_enabled: true,
            animations_enabled: true,
            file_context_enabled: true,
            session_directory_filter_enabled: true,
            diff_wrap_mode: "word".to_string(),
            permission_mode_auto: false,
            ready: false,
            plugin_ready: false,
            session_id: None,
            runner_rx: None,
            acp_rx: None,
            current_assistant_text: String::new(),
            current_reasoning_text: String::new(),
            system_prompt: "You are a helpful AI coding assistant. Use tools when needed to accomplish tasks.".to_string(),
            store: default_store(),
            current_model: String::new(),
            total_input_tokens: 0,
            total_output_tokens: 0,
            tool_call_count: 0,
            step_count: 0,
            message_queue: Vec::new(),
        }
    }

    pub fn set_theme(&mut self, name: &str) {
        self.theme = theme::get_theme(name);
    }

    /// Update the current model/provider display. Propagates to the prompt
    /// meta line and the App.current_model field (used by assistant footers).
    pub fn set_model(&mut self, model: impl Into<String>, provider: impl Into<String>) {
        let model = model.into();
        let provider = provider.into();
        self.current_model = model.clone();
        self.prompt.model = model;
        self.prompt.provider = provider;
    }

    /// Resume a previous session by loading its messages from the store
    /// and rebuilding the TUI message list. If the session doesn't exist
    /// (e.g. id typo, or store is in-memory), a toast is shown.
    pub fn resume_session(&mut self, session_id: &str) {
        use crate::core::session::SessionStore;
        let sid = crate::schema::ids::SessionID(session_id.to_string());
        let store = self.store.clone();

        // We can't await in a sync method, so spawn a task that fetches
        // the messages and pushes them into the App via a channel. For
        // simplicity, block_on a quick fetch (SqliteSessionStore is fast).
        let store_clone = store.clone();
        let sid_clone = sid.clone();
        let messages = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                store_clone.context(&sid_clone).await
            })
        });

        match messages {
            Some(msgs) if !msgs.is_empty() => {
                self.session_id = Some(sid);
                self.messages.clear();
                self.messages.push(crate::tui::app::ChatMessage::new(
                    crate::tui::app::MessageRole::System,
                    crate::t!("tui.welcome").to_string(),
                ));
                for m in msgs {
                    use crate::schema::session::SessionMessage;
                    match m {
                        SessionMessage::User { text, .. } => {
                            self.messages.push(crate::tui::app::ChatMessage::new(
                                crate::tui::app::MessageRole::User,
                                text,
                            ));
                        }
                        SessionMessage::Assistant { content, .. } => {
                            let mut msg = crate::tui::app::ChatMessage::new(
                                crate::tui::app::MessageRole::Assistant,
                                String::new(),
                            );
                            for part in content {
                                if let crate::schema::session::AssistantContent::Text { text, .. } = part {
                                    msg.push_text(text);
                                }
                            }
                            self.messages.push(msg);
                        }
                        SessionMessage::System { text, .. } => {
                            self.messages.push(crate::tui::app::ChatMessage::new(
                                crate::tui::app::MessageRole::System,
                                text,
                            ));
                        }
                        _ => {}
                    }
                }
                self.scroll_to_bottom();
                self.toast_manager.show(
                    format!("Resumed session {}", &session_id[..session_id.len().min(16)]),
                    crate::tui::component::toast::ToastVariant::Info,
                );
            }
            _ => {
                self.toast_manager.show(
                    format!("Session not found: {}", session_id),
                    crate::tui::component::toast::ToastVariant::Error,
                );
            }
        }
    }

    pub fn set_route(&mut self, route: Route) {
        self.route = route;
        self.update_terminal_title();
    }

    pub fn exit(&mut self) {
        self.should_quit = true;
        self.runtime_state.destroy();
        // Stash the session id so run() can print a resume hint on exit.
        if let Some(sid) = &self.session_id {
            self.exit_state.reason = Some(format!("__resume__:{}", sid.0));
        }
    }

    pub fn exit_with_reason(&mut self, reason: String) {
        self.exit_state.reason = Some(reason);
        self.exit();
    }

    pub fn update_terminal_title(&self) {
        if !self.terminal_title_enabled || env::var("OPENCODE_DISABLE_TERMINAL_TITLE").is_ok() {
            return;
        }
        match &self.route {
            Route::Home => runtime::set_terminal_title("OpenCode"),
            Route::Session { session_id: _ } => {
                runtime::set_terminal_title("OpenCode");
            }
            Route::Plugin { id, .. } => {
                runtime::set_terminal_title(&format!("OC | {}", id));
            }
        }
    }

    /// Whether the view is currently pinned to the bottom (within 1 line of
    /// max_scroll). Delegates to the ScrollView.
    pub fn is_at_bottom(&self) -> bool {
        self.messages_scroll.is_at_bottom()
    }

    /// Follow new content only if the user is already at the bottom. This is
    /// the sticky-scroll behavior: scrolling up to read history won't be
    /// disrupted by incoming messages.
    pub fn follow_if_at_bottom(&mut self) {
        self.messages_scroll.follow_if_at_bottom();
    }

    /// Pin to the bottom unconditionally (e.g. on user submit / resume).
    pub fn scroll_to_bottom(&mut self) {
        self.messages_scroll.scroll_to_bottom();
    }

    /// Toggle the vertical scrollbar on the message list.
    pub fn toggle_scrollbar(&mut self) {
        self.messages_scroll.toggle_scrollbar();
    }

    pub fn submit_message(&mut self, text: String) {
        let normalized = text.replace("\r\n", "\n").replace('\r', "\n");
        let trimmed = normalized.trim().to_string();
        if trimmed.is_empty() {
            return;
        }
        // If the LLM is still generating, show the message immediately but
        // mark it as queued. When the current turn finishes (RunnerEvent::Done),
        // the queued messages are promoted to active one at a time.
        // This mirrors the original TS behavior where queued messages appear
        // in the chat with a `QUEUED` tag.
        if self.is_thinking {
            let mut msg = ChatMessage::new(MessageRole::User, trimmed.clone());
            msg.queued = true;
            self.messages.push(msg);
            self.message_queue.push(trimmed.clone());
            self.scroll_to_bottom();
            self.prompt.clear();
            return;
        }
        // Slash commands: intercept before sending to LLM.
        if trimmed.starts_with('/') {
            let cmd = trimmed.trim_start_matches('/').split_whitespace().next().unwrap_or("");
            let args = trimmed.trim_start_matches('/').strip_prefix(cmd).unwrap_or("").trim().to_string();
            self.prompt.clear();
            self.handle_slash_command(cmd, args);
            return;
        }
        // Check if the LAST message is the same (promoted from queue).
        // Only check the last one to allow duplicate user messages.
        let already_present = self.messages.last().map_or(false, |m| {
            m.role == MessageRole::User && m.text == trimmed && !m.queued
        });
        if !already_present {
            self.messages.push(ChatMessage::new(MessageRole::User, trimmed.clone()));
        }
        self.scroll_to_bottom();
        self.prompt.clear();
        self.is_thinking = true;
        self.footer.status = crate::t!("tui.status.thinking").to_string();
        self.spinner_active = true;
        // Reset spinner to thinking mode + pick a new random verb.
        self.spinner.set_mode(crate::tui::component::spinner::SpinnerMode::Thinking);
        self.spinner.pick_new_verb();
        dbg_log(&format!("submit_message: text={:?}", trimmed));

        let selection = match provider_factory::select_from_env() {
            Some(s) => { dbg_log(&format!("provider selected: id={} model={}", s.provider_id, s.model_id)); s }
            None => {
                self.messages.push(ChatMessage::new(
                    MessageRole::Assistant,
                    crate::t!("tui.error.no_api_key").to_string(),
                ));
                self.is_thinking = false;
                self.spinner_active = false;
                self.footer.status = crate::t!("tui.status.idle").to_string();
                return;
            }
        };

        // Sync the prompt's model/provider display so the user sees what's in use.
        self.set_model(selection.model_name.clone(), selection.provider_name.clone());
        dbg_log("after set_model, before session_id");

        let session_id = self.session_id.clone().unwrap_or_else(|| {
            let id = SessionID::new();
            self.session_id = Some(id.clone());
            id
        });

        // Ensure the session exists in the store (runner looks it up by id).
        // The runner also does this defensively, so we just let the runner
        // create the stub if needed.

        let (tx, rx) = mpsc::channel::<RunnerEvent>(256);
        // ACP bridge (claude-code-book Ch02/Ch13): drain RunnerEvent and
        // forward as AcpEvent. TUI subscribes to acp_rx — fully decoupled
        // from the RunnerEvent type.
        let (acp_tx, acp_rx) = mpsc::channel::<crate::core::acp::AcpEvent>(256);
        self.acp_rx = Some(acp_rx);
        crate::core::acp::spawn_bridge(rx, acp_tx);
        self.runner_rx = None; // rx consumed by bridge; TUI reads acp_rx
        let tx_send = tx.clone();

        let store = self.store.clone();
        let session_id_clone = session_id.clone();
        let user_text = trimmed.to_string();
        let system_prompt = self.system_prompt.clone();

        let model_resolver = Arc::new(crate::core::session::runner::model::EnvModelResolver::new(
            selection.model_id,
            selection.provider_id,
        ));
        let tools = Arc::new(crate::tools::registry::ToolRegistry::builtin());
        let provider = selection.provider;
        let agent_id = "build".to_string();

        let runner = Arc::new(SessionRunner::new(provider, tools, store.clone(), model_resolver));
        dbg_log("runner created, about to spawn");

        // Spawn a single task that:
        // 1. Appends the user message to the session store
        // 2. Runs the agent loop
        // Doing both in the same task guarantees the message is stored
        // before the runner tries to read it.
        tokio::spawn(async move {
            // Append the user message first (ensure session exists first).
            {
                use crate::schema::session::{SessionInfo, SessionTokens, SessionTime, SessionMessage};
                use crate::schema::ids::{SessionMessageID, ProjectID, AgentID};
                use crate::schema::location::LocationRef;
                use crate::schema::common::AbsolutePath;
                // Create session stub if it doesn't exist.
                if store.get(&session_id_clone).await.is_none() {
                    let cwd = std::env::current_dir()
                        .map(|p| AbsolutePath(p.to_string_lossy().to_string()))
                        .unwrap_or_else(|_| AbsolutePath(String::from("/")));
                    let info = SessionInfo {
                        id: session_id_clone.clone(),
                        parent_id: None,
                        project_id: ProjectID::from_str("default"),
                        agent: Some(AgentID("build".to_string())),
                        model: None,
                        cost: 0.0,
                        tokens: SessionTokens::default(),
                        time: SessionTime {
                            created: chrono::Utc::now(),
                            updated: chrono::Utc::now(),
                            archived: None,
                        },
                        title: crate::core::session::default_parent_title(),
                        location: LocationRef {
                            directory: cwd,
                            workspace_id: None,
                        },
                        subpath: None,
                        revert: None,
                    };
                    store.create(info).await;
                    dbg_log("session stub created before append");
                }
                let msg = SessionMessage::User {
                    id: SessionMessageID::new(),
                    metadata: None,
                    time: crate::schema::session::MessageTime {
                        created: chrono::Utc::now(),
                    },
                    text: user_text.clone(),
                    files: None,
                    agents: None,
                };
                let appended = store.append_message(&session_id_clone, msg).await;
                dbg_log(&format!("user message appended: {}", appended));
            }

            dbg_log("runner spawned, calling run_with_events");
            let result = runner
                .run_with_events(&session_id_clone, &system_prompt, &agent_id, None, tx)
                .await;
            match &result {
                Ok(r) => dbg_log(&format!("runner finished: steps={}", r.steps)),
                Err(e) => {
                    dbg_log(&format!("runner error: {}", e));
                    // CRITICAL: send Error event to TUI so is_thinking is reset.
                    // Without this, the TUI stays in "thinking" forever and
                    // queued messages are never flushed.
                    let _ = tx_send.send(crate::core::session::runner::RunnerEvent::Error {
                        message: e.to_string(),
                    }).await;
                }
            }
            if let Err(e) = result {
                tracing::error!("Session runner error: {}", e);
            }
        });
    }

    /// Handle a slash command (e.g. `/new`, `/help`, `/exit`).
    pub fn handle_slash_command(&mut self, cmd: &str, args: String) {
        match cmd {
            "new" => {
                self.messages.clear();
                self.messages.push(ChatMessage::new(MessageRole::System, crate::t!("tui.welcome").to_string()));
                self.session_id = None;
                self.total_input_tokens = 0;
                self.total_output_tokens = 0;
                self.tool_call_count = 0;
                self.step_count = 0;
                self.sidebar.context_tokens = 0;
                self.sidebar.step_count = 0;
                self.sidebar.tool_call_count = 0;
                self.sidebar.context_cost = 0.0;
                self.route = Route::Home;
                self.toast_manager.show(crate::t!("tui.session.new_started").to_string(), crate::tui::component::toast::ToastVariant::Info);
            }
            "clear" => {
                self.messages.clear();
                self.messages.push(ChatMessage::new(MessageRole::System, crate::t!("tui.welcome").to_string()));
                self.scroll_to_bottom();
            }
            "help" => {
                self.mode = InputMode::Help;
            }
            "exit" | "quit" => {
                self.exit();
            }
            "themes" | "theme" => {
                // /theme <name> → switch directly; /themes → cycle to next.
                if !args.is_empty() {
                    let name = args.trim();
                    let names = crate::tui::theme::loader::list_theme_names();
                    if names.iter().any(|n| *n == name) {
                        self.set_theme(name);
                        self.toast_manager.show(format!("Theme: {}", name), crate::tui::component::toast::ToastVariant::Info);
                    } else {
                        self.toast_manager.show(format!("Unknown theme: {} ({} available)", name, names.len()), crate::tui::component::toast::ToastVariant::Error);
                    }
                } else {
                    // Cycle to the next theme.
                    let names = crate::tui::theme::loader::list_theme_names();
                    let current_idx = names.iter().position(|n| *n == "opencode").unwrap_or(0);
                    let next_idx = (current_idx + 1) % names.len();
                    if let Some(name) = names.get(next_idx) {
                        self.set_theme(name);
                        self.toast_manager.show(format!("Theme: {} (use /theme <name>)", name), crate::tui::component::toast::ToastVariant::Info);
                    }
                }
            }
            "status" => {
                let info = format!(
                    "session: {} | steps: {} | tools: {} | tokens: {}",
                    self.session_id.as_ref().map(|s| s.0.as_str()).unwrap_or("none"),
                    self.step_count,
                    self.tool_call_count,
                    self.total_input_tokens + self.total_output_tokens,
                );
                self.toast_manager.show(info, crate::tui::component::toast::ToastVariant::Info);
            }
            "model" | "models" => {
                use crate::llm::provider_factory;
                let models = provider_factory::list_configured_models();
                if models.is_empty() {
                    self.toast_manager.show(
                        "No models configured. Edit ~/.rsopencode/config.toml".to_string(),
                        crate::tui::component::toast::ToastVariant::Warning,
                    );
                } else if !args.is_empty() {
                    // /model <id> — switch directly.
                    let target = args.trim();
                    if let Some(name) = provider_factory::switch_model(target) {
                        self.set_model(name.clone(), String::new());
                        // Update the config file's default_model.
                        if let Some(home) = dirs::home_dir() {
                            let path = home.join(".rsopencode").join("config.toml");
                            if let Ok(content) = std::fs::read_to_string(&path) {
                                let updated = update_config_default_model(&content, target);
                                let _ = std::fs::write(&path, updated);
                            }
                        }
                        self.toast_manager.show(
                            format!("Switched to: {}", name),
                            crate::tui::component::toast::ToastVariant::Success,
                        );
                    } else {
                        let available: Vec<_> = models.iter().map(|(_, id, _)| id.clone()).collect();
                        self.toast_manager.show(
                            format!("Unknown model: {} (available: {})", target, available.join(", ")),
                            crate::tui::component::toast::ToastVariant::Error,
                        );
                    }
                } else {
                    // /model with no args — open a selection dialog.
                    use crate::tui::component::dialog::{Dialog, DialogOption};
                    let options: Vec<DialogOption> = models.iter().map(|(provider_id, id, name)| {
                        let title = if id == name { id.clone() } else { format!("{} ({})", name, id) };
                        DialogOption::new(title, id.clone())
                            .with_description(format!("provider: {}", provider_id))
                    }).collect();
                    if !options.is_empty() {
                        self.dialog = Some(Dialog::select("Select Model", options));
                    }
                }
            }
            // Commands that need a dialog or external action — show a toast for now.
            "sessions" | "workspaces" | "agents" | "mcps" | "variants"
            | "connect" | "org" | "debug" | "editor" | "skills" | "warp" | "move" | "diff" => {
                self.toast_manager.show(format!("/{cmd} — not yet implemented (args: {})", args), crate::tui::component::toast::ToastVariant::Info);
            }
            _ => {
                self.toast_manager.show(format!("Unknown command: /{}", cmd), crate::tui::component::toast::ToastVariant::Info);
            }
        }
    }

    pub fn toggle_terminal_title(&mut self) {
        self.terminal_title_enabled = !self.terminal_title_enabled;
        if !self.terminal_title_enabled {
            runtime::clear_terminal_title();
        } else {
            self.update_terminal_title();
        }
    }

    pub fn toggle_animations(&mut self) {
        self.animations_enabled = !self.animations_enabled;
    }

    pub fn toggle_file_context(&mut self) {
        self.file_context_enabled = !self.file_context_enabled;
    }

    pub fn toggle_diff_wrap(&mut self) {
        self.diff_wrap_mode = if self.diff_wrap_mode == "word" {
            "none".to_string()
        } else {
            "word".to_string()
        };
    }

    pub fn toggle_paste_summary(&mut self) {
        self.paste_summary_enabled = !self.paste_summary_enabled;
    }

    pub fn toggle_session_directory_filter(&mut self) {
        self.session_directory_filter_enabled = !self.session_directory_filter_enabled;
    }

    pub fn toggle_permission_mode(&mut self) {
        self.permission_mode_auto = !self.permission_mode_auto;
    }

    pub fn poll_runner_events(&mut self) {
        // claude-code-book Ch02/Ch13: TUI subscribes to AcpEvent, not RunnerEvent.
        // The bridge task drains the runner channel and forwards converted events.
        let rx = match &mut self.acp_rx {
            Some(rx) => rx,
            None => return,
        };

        while let Ok(event) = rx.try_recv() {
            use crate::core::acp::{AcpEvent, StreamDelta};
            match event {
                AcpEvent::StreamRequestStart { step, .. } => {
                    self.current_assistant_text.clear();
                    self.current_reasoning_text.clear();
                    let needs_new = match self.messages.last() {
                        None => true,
                        Some(last) => last.role != MessageRole::Assistant,
                    };
                    if needs_new {
                        self.messages.push(ChatMessage::new(MessageRole::Assistant, String::new()));
                    }
                    self.messages_scroll.follow_if_at_bottom();
                    self.step_count = step;
                    self.sidebar.context_tokens = self.total_input_tokens + self.total_output_tokens;
                    self.sidebar.tool_call_count = self.tool_call_count;
                    self.sidebar.step_count = self.step_count;
                    self.spinner.set_mode(crate::tui::component::spinner::SpinnerMode::Thinking);
                    self.spinner.pick_new_verb();
                }
                AcpEvent::StreamEvent(StreamDelta::Text { text }) => {
                    self.spinner.set_mode(crate::tui::component::spinner::SpinnerMode::Responding);
                    self.current_assistant_text.push_str(&text);
                    if let Some(last) = self.messages.last_mut() {
                        if last.role == MessageRole::Assistant {
                            last.push_text(text);
                        }
                    }
                }
                AcpEvent::StreamEvent(StreamDelta::Reasoning { text }) => {
                    self.current_reasoning_text.push_str(&text);
                    if let Some(last) = self.messages.last_mut() {
                        if last.role == MessageRole::Assistant {
                            last.push_text(text);
                        }
                    }
                }
                AcpEvent::TextEnd => {
                    self.current_assistant_text.clear();
                }
                AcpEvent::ReasoningEnd => {
                    self.current_reasoning_text.clear();
                    if let Some(last) = self.messages.last_mut() {
                        if last.role == MessageRole::Assistant {
                            last.push_text("\n");
                        }
                    }
                }
                AcpEvent::ToolStarted { tool_name, call_id, input } => {
                    let state = ToolPartState::Pending { input };
                    self.tool_call_count += 1;
                    self.sidebar.tool_call_count = self.tool_call_count;
                    self.footer.status = format!("▶ {}", tool_display_name(&tool_name));
                    self.spinner_active = true;
                    self.spinner.set_mode(crate::tui::component::spinner::SpinnerMode::ToolUse);
                    if let Some(last) = self.messages.last_mut() {
                        if last.role == MessageRole::Assistant {
                            last.push_tool(tool_name, call_id, state);
                            self.messages_scroll.follow_if_at_bottom();
                            continue;
                        }
                    }
                    let mut msg = ChatMessage::new(MessageRole::Assistant, String::new());
                    msg.push_tool(tool_name, call_id, state);
                    self.messages.push(msg);
                    self.messages_scroll.follow_if_at_bottom();
                }
                AcpEvent::ToolSuccess { tool_name, call_id, summary } => {
                    let new_state = ToolPartState::Completed {
                        input: serde_json::Value::Null,
                        output: summary,
                    };
                    let updated = self
                        .messages
                        .last_mut()
                        .map(|m| m.complete_tool(&call_id, new_state.clone()))
                        .unwrap_or(false);
                    if !updated {
                        let mut msg = ChatMessage::new(MessageRole::Assistant, String::new());
                        msg.push_tool(tool_name, call_id, new_state);
                        self.messages.push(msg);
                    } else {
                        refresh_message_text(self.messages.last_mut().unwrap());
                    }
                    self.messages_scroll.follow_if_at_bottom();
                    self.footer.status = crate::t!("tui.status.thinking").to_string();
                    self.spinner.set_mode(crate::tui::component::spinner::SpinnerMode::Thinking);
                    self.spinner.pick_new_verb();
                }
                AcpEvent::ToolFailed { tool_name, call_id, error } => {
                    let new_state = ToolPartState::Error {
                        input: serde_json::Value::Null,
                        error: error.clone(),
                    };
                    let updated = self
                        .messages
                        .last_mut()
                        .map(|m| m.complete_tool(&call_id, new_state.clone()))
                        .unwrap_or(false);
                    if !updated {
                        let mut msg = ChatMessage::new(MessageRole::Assistant, String::new());
                        msg.push_tool(tool_name, call_id, new_state);
                        self.messages.push(msg);
                    } else {
                        refresh_message_text(self.messages.last_mut().unwrap());
                    }
                    self.messages_scroll.follow_if_at_bottom();
                    self.footer.status = crate::t!("tui.status.thinking").to_string();
                    self.spinner.set_mode(crate::tui::component::spinner::SpinnerMode::Thinking);
                    self.spinner.pick_new_verb();
                }
                AcpEvent::StepFinished { usage, .. } => {
                    if let Some(u) = usage {
                        self.total_input_tokens += u.input_tokens.unwrap_or(0);
                        self.total_output_tokens += u.output_tokens.unwrap_or(0);
                        let in_cost = (u.input_tokens.unwrap_or(0) as f64) * 3.0 / 1_000_000.0;
                        let out_cost = (u.output_tokens.unwrap_or(0) as f64) * 15.0 / 1_000_000.0;
                        self.sidebar.context_cost += in_cost + out_cost;
                    }
                    self.sidebar.context_tokens = self.total_input_tokens + self.total_output_tokens;
                    self.sidebar.tool_call_count = self.tool_call_count;
                    self.sidebar.step_count = self.step_count;
                }
                AcpEvent::CompactionNeeded { tier, used, effective } => {
                    use crate::core::session::compaction::CompactionTier;
                    let pct = if effective > 0 {
                        (used as f64 / effective as f64 * 100.0) as u64
                    } else { 0 };
                    let (msg, variant) = match tier {
                        CompactionTier::Warning => (
                            format!("Context at {}% — consider /compact", pct),
                            crate::tui::component::toast::ToastVariant::Warning,
                        ),
                        CompactionTier::AutoCompact => (
                            format!("Context at {}% — auto-compacting", pct),
                            crate::tui::component::toast::ToastVariant::Warning,
                        ),
                        CompactionTier::Blocking => (
                            format!("Context at {}% — blocked, compact needed", pct),
                            crate::tui::component::toast::ToastVariant::Error,
                        ),
                        CompactionTier::None => (String::new(), crate::tui::component::toast::ToastVariant::Info),
                    };
                    if !msg.is_empty() {
                        self.toast_manager.show(msg, variant);
                    }
                }
                AcpEvent::Error { message } => {
                    self.messages.push(ChatMessage::new(
                        MessageRole::System,
                        crate::t!("tui.message.error", message = message).to_string(),
                    ));
                    self.messages_scroll.follow_if_at_bottom();
                    self.is_thinking = false;
                    self.spinner_active = false;
                    self.footer.status = crate::t!("tui.status.error").to_string();
                    self.acp_rx = None;
                    return;
                }
                AcpEvent::Done { .. } => {
                    self.is_thinking = false;
                    self.spinner_active = false;
                    self.footer.status = crate::t!("tui.status.idle").to_string();
                    self.acp_rx = None;
                    // Promote the next queued message: find the first queued
                    // user message, un-queue it, and re-submit it so it
                    // actually gets sent to the LLM.
                    if !self.message_queue.is_empty() {
                        // Remove from the queue.
                        let next = self.message_queue.remove(0);
                        // Find the matching queued message in the chat and
                        // un-queue it (so the QUEUED tag disappears).
                        for msg in self.messages.iter_mut() {
                            if msg.queued && msg.role == MessageRole::User && msg.text == next {
                                msg.queued = false;
                                break;
                            }
                        }
                        // Now actually submit it (is_thinking is false so it
                        // will go through to the LLM).
                        self.submit_message(next);
                    }
                    return;
                }
            }
        }
    }

    pub fn open_external_editor(&mut self) {
        let value = self.prompt.input.clone();
        match editor::open_editor(&value, None) {
            Ok(Some(content)) => {
                let normalized = editor::normalize_prompt_content(&content);
                self.prompt.input = normalized;
                self.prompt.focused = true;
                self.toast_manager.show(
                    crate::t!("tui.editor.loaded"),
                    crate::tui::component::toast::ToastVariant::Info,
                );
            }
            Ok(None) => {
                self.toast_manager.show(
                    crate::t!("tui.editor.empty"),
                    crate::tui::component::toast::ToastVariant::Info,
                );
            }
            Err(e) => {
                self.toast_manager.show(
                    crate::t!("tui.editor.error", error = e).to_string(),
                    crate::tui::component::toast::ToastVariant::Error,
                );
            }
        }
    }

    pub fn suspend_terminal(&mut self) {
        #[cfg(unix)]
        {
            runtime::suspend_terminal();
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a default session store. Uses JSON files at
/// `~/.rsopencode/sessions/<id>.json` for persistence across restarts —
/// each session is a human-readable, easy-to-edit file. Falls back to
/// in-memory if the dir can't be created.
/// Update the `default_model` line in a config.toml string. If the line
/// doesn't exist, it's inserted after `default_provider`.
pub fn update_config_default_model(config: &str, model_id: &str) -> String {
    let mut lines: Vec<String> = config.lines().map(|l| l.to_string()).collect();
    let mut found = false;
    for line in &mut lines {
        let trimmed = line.trim();
        if trimmed.starts_with("default_model") {
            *line = format!("default_model = \"{}\"", model_id);
            found = true;
            break;
        }
    }
    if !found {
        // Insert after default_provider, or at the top.
        let mut insert_at = 0;
        for (i, line) in lines.iter().enumerate() {
            if line.trim().starts_with("default_provider") {
                insert_at = i + 1;
                break;
            }
        }
        lines.insert(insert_at, format!("default_model = \"{}\"", model_id));
    }
    lines.join("\n") + "\n"
}

/// Translate a tool name to a localized display name for the footer status.
fn tool_display_name(name: &str) -> String {
    let key = format!("tui.tool.{}", name.to_lowercase());
    let translated = crate::tui::i18n::t(&key);
    if translated == key {
        // No translation found — return the original name.
        name.to_string()
    } else {
        translated
    }
}

fn default_store() -> Arc<dyn crate::core::session::SessionStore> {
    use crate::core::session::json_store::JsonSessionStore;
    if let Some(dir) = JsonSessionStore::default_dir() {
        match JsonSessionStore::new(&dir) {
            Ok(s) => return Arc::new(s),
            Err(e) => {
                crate::tui::app::util::dbg_log(&format!("json store failed, using in-memory: {}", e));
            }
        }
    }
    Arc::new(crate::core::session::store::InMemorySessionStore::new())
}
