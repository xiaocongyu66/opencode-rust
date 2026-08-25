use super::*;

    #[test]
    fn test_is_version_greater_basic() {
        assert!(is_version_greater("1.0.1", "1.0.0"));
        assert!(!is_version_greater("1.0.0", "1.0.1"));
        assert!(!is_version_greater("1.0.0", "1.0.0"));
    }

    #[test]
    fn test_is_version_greater_major() {
        assert!(is_version_greater("2.0.0", "1.9.9"));
        assert!(!is_version_greater("1.9.9", "2.0.0"));
    }

    #[test]
    fn test_is_version_greater_v_prefix() {
        assert!(is_version_greater("v1.0.1", "1.0.0"));
        assert!(is_version_greater("1.0.1", "v1.0.0"));
    }

    #[test]
    fn test_is_version_greater_prerelease() {
        assert!(is_version_greater("1.0.0", "1.0.0-beta"));
        assert!(!is_version_greater("1.0.0-beta", "1.0.0"));
    }

    #[test]
    fn test_is_version_greater_different_lengths() {
        assert!(is_version_greater("1.2.3.4", "1.2.3"));
        assert!(!is_version_greater("1.2.3", "1.2.3.4"));
    }

    #[test]
    fn test_parse_version_simple() {
        let (core, pre) = parse_version("1.2.3");
        assert_eq!(core, vec![1, 2, 3]);
        assert_eq!(pre, None);
    }

    #[test]
    fn test_parse_version_v_prefix() {
        let (core, pre) = parse_version("v2.0.0");
        assert_eq!(core, vec![2, 0, 0]);
        assert_eq!(pre, None);
    }

    #[test]
    fn test_parse_version_prerelease() {
        let (core, pre) = parse_version("1.0.0-beta.1");
        assert_eq!(core, vec![1, 0, 0]);
        assert_eq!(pre, Some("beta.1".to_string()));
    }

    #[test]
    fn test_parse_version_invalid_parts() {
        let (core, _) = parse_version("1.x.3");
        assert_eq!(core, vec![1, 0, 3]);
    }

    #[test]
    fn test_error_message_from_data() {
        let val = serde_json::json!({"data": {"message": "test error"}});
        assert_eq!(error_message(&val), "test error");
    }

    #[test]
    fn test_error_message_from_root() {
        let val = serde_json::json!({"message": "root error"});
        assert_eq!(error_message(&val), "root error");
    }

    #[test]
    fn test_error_message_fallback() {
        let val = serde_json::json!({"foo": "bar"});
        assert_eq!(error_message(&val), r#"{"foo":"bar"}"#);
    }

    #[test]
    fn test_app_new() {
        let app = App::new();
        assert!(!app.should_quit);
        assert_eq!(app.mode, InputMode::Insert);
        assert!(!app.messages.is_empty());
        assert!(app.terminal_title_enabled);
        assert!(app.animations_enabled);
        assert!(app.file_context_enabled);
    }

    #[test]
    fn test_app_exit() {
        let mut app = App::new();
        app.exit();
        assert!(app.should_quit);
        assert!(app.runtime_state.destroyed);
    }

    #[test]
    fn test_app_exit_with_reason() {
        let mut app = App::new();
        app.exit_with_reason("test reason".to_string());
        assert!(app.should_quit);
        assert_eq!(app.exit_state.reason, Some("test reason".to_string()));
    }

    #[test]
    fn test_app_toggle_terminal_title() {
        let mut app = App::new();
        assert!(app.terminal_title_enabled);
        app.toggle_terminal_title();
        assert!(!app.terminal_title_enabled);
        app.toggle_terminal_title();
        assert!(app.terminal_title_enabled);
    }

    #[test]
    fn test_app_toggle_animations() {
        let mut app = App::new();
        assert!(app.animations_enabled);
        app.toggle_animations();
        assert!(!app.animations_enabled);
    }

    #[test]
    fn test_app_toggle_file_context() {
        let mut app = App::new();
        assert!(app.file_context_enabled);
        app.toggle_file_context();
        assert!(!app.file_context_enabled);
    }

    #[test]
    fn test_app_toggle_diff_wrap() {
        let mut app = App::new();
        assert_eq!(app.diff_wrap_mode, "word");
        app.toggle_diff_wrap();
        assert_eq!(app.diff_wrap_mode, "none");
        app.toggle_diff_wrap();
        assert_eq!(app.diff_wrap_mode, "word");
    }

    #[test]
    fn test_app_toggle_paste_summary() {
        let mut app = App::new();
        assert!(app.paste_summary_enabled);
        app.toggle_paste_summary();
        assert!(!app.paste_summary_enabled);
    }

    #[test]
    fn test_app_toggle_session_directory_filter() {
        let mut app = App::new();
        assert!(app.session_directory_filter_enabled);
        app.toggle_session_directory_filter();
        assert!(!app.session_directory_filter_enabled);
    }

    #[test]
    fn test_app_toggle_permission_mode() {
        let mut app = App::new();
        assert!(!app.permission_mode_auto);
        app.toggle_permission_mode();
        assert!(app.permission_mode_auto);
    }

    #[test]
    fn test_app_set_route() {
        let mut app = App::new();
        app.set_route(Route::Session {
            session_id: "test-123".to_string(),
        });
        match &app.route {
            Route::Session { session_id } => assert_eq!(session_id, "test-123"),
            _ => panic!("route should be Session"),
        }
    }

    #[test]
    fn test_app_submit_message_empty() {
        let mut app = App::new();
        let initial_len = app.messages.len();
        app.submit_message("".to_string());
        assert_eq!(app.messages.len(), initial_len);
    }

    #[test]
    fn test_app_submit_message_valid() {
        let mut app = App::new();
        app.submit_message("hello world".to_string());
        assert!(app.messages.iter().any(|m| m.text == "hello world"));
    }

    #[test]
    fn test_handle_update_available_not_skipped() {
        let mut app = App::new();
        assert!(handle_update_available(&mut app, "1.0.1", None));
    }

    #[test]
    fn test_handle_update_available_skipped_lower() {
        let mut app = App::new();
        assert!(!handle_update_available(&mut app, "1.0.0", Some("1.0.1")));
    }

    #[test]
    fn test_handle_update_available_skipped_higher() {
        let mut app = App::new();
        assert!(handle_update_available(&mut app, "1.0.2", Some("1.0.1")));
    }

    #[test]
    fn test_handle_session_deleted_current() {
        let mut app = App::new();
        app.set_route(Route::Session {
            session_id: "test-123".to_string(),
        });
        handle_session_deleted(&mut app, "test-123");
        assert!(matches!(app.route, Route::Home));
    }

    #[test]
    fn test_handle_session_deleted_other() {
        let mut app = App::new();
        app.set_route(Route::Session {
            session_id: "test-123".to_string(),
        });
        handle_session_deleted(&mut app, "other-456");
        assert!(matches!(app.route, Route::Session { .. }));
    }

    #[test]
    fn test_handle_session_select() {
        let mut app = App::new();
        handle_session_select(&mut app, "session-789");
        match &app.route {
            Route::Session { session_id } => assert_eq!(session_id, "session-789"),
            _ => panic!("route should be Session"),
        }
    }

    #[test]
    fn test_handle_session_error_aborted() {
        let mut app = App::new();
        let error = serde_json::json!({"name": "MessageAbortedError"});
        handle_session_error(&mut app, &error);
    }

    #[test]
    fn test_handle_session_error_normal() {
        let mut app = App::new();
        let error = serde_json::json!({"name": "SomeError", "message": "something broke"});
        handle_session_error(&mut app, &error);
    }

    #[test]
    fn test_handle_copy_selection_empty() {
        let mut app = App::new();
        handle_copy_selection(&mut app, "");
    }

    #[test]
    fn test_handle_copy_selection_text() {
        let mut app = App::new();
        handle_copy_selection(&mut app, "hello world");
    }

    #[test]
    fn test_constants_app_global_binding_commands() {
        assert_eq!(APP_GLOBAL_BINDING_COMMANDS.len(), 11);
        assert!(APP_GLOBAL_BINDING_COMMANDS.contains(&"session.list"));
        assert!(APP_GLOBAL_BINDING_COMMANDS.contains(&"session.new"));
    }

    #[test]
    fn test_constants_app_binding_commands() {
        assert_eq!(APP_BINDING_COMMANDS.len(), 32);
        assert!(APP_BINDING_COMMANDS.contains(&"command.palette.show"));
        assert!(APP_BINDING_COMMANDS.contains(&"model.list"));
        assert!(APP_BINDING_COMMANDS.contains(&"theme.switch"));
    }

    #[test]
    fn test_constants_tokens() {
        assert_eq!(LEADER_TOKEN, "leader");
        assert_eq!(OPENCODE_BASE_MODE, "base");
        assert_eq!(COMMAND_PALETTE_COMMAND, "command.palette.show");
    }

    #[test]
    fn test_route_default() {
        assert!(matches!(Route::default(), Route::Home));
    }

    #[test]
    fn test_tui_input_default() {
        let input = TuiInput::default();
        assert!(input.url.is_empty());
        assert!(input.directory.is_none());
    }

    #[test]
    fn test_exit_state_default() {
        let state = ExitState::default();
        assert!(state.epilogue.is_none());
        assert!(state.reason.is_none());
    }

    #[test]
    fn chat_message_new_starts_with_empty_parts() {
        let m = ChatMessage::new(MessageRole::User, "hi");
        assert_eq!(m.role, MessageRole::User);
        assert_eq!(m.text, "hi");
        assert!(m.parts.is_empty());
    }

    #[test]
    fn push_text_accumulates_into_parts_and_text() {
        let mut m = ChatMessage::new(MessageRole::Assistant, String::new());
        m.push_text("hello ");
        m.push_text("world");
        assert_eq!(m.text, "hello world");
        assert_eq!(m.parts.len(), 2);
        assert_eq!(m.parts[0].as_text(), Some("hello "));
        assert_eq!(m.parts[1].as_text(), Some("world"));
    }

    #[test]
    fn push_tool_adds_pending_part() {
        let mut m = ChatMessage::new(MessageRole::Assistant, String::new());
        let input = serde_json::json!({"command": "ls"});
        m.push_tool(
            "bash".to_string(),
            "call_1".to_string(),
            ToolPartState::Pending { input: input.clone() },
        );
        assert_eq!(m.parts.len(), 1);
        match &m.parts[0] {
            ChatPart::Tool { tool_name, call_id, state } => {
                assert_eq!(tool_name, "bash");
                assert_eq!(call_id, "call_1");
                assert!(matches!(state, ToolPartState::Pending { .. }));
                assert_eq!(state.input(), &input);
                assert!(!state.is_terminal());
            }
            other => panic!("expected Tool, got {other:?}"),
        }
        // Legacy text field should contain a summary.
        assert!(m.text.contains("bash"));
    }

    #[test]
    fn complete_tool_transitions_pending_to_completed() {
        let mut m = ChatMessage::new(MessageRole::Assistant, String::new());
        m.push_tool(
            "read".to_string(),
            "call_2".to_string(),
            ToolPartState::Pending { input: serde_json::json!({"path": "/x"}) },
        );
        let updated = m.complete_tool(
            "call_2",
            ToolPartState::Completed {
                input: serde_json::Value::Null,
                output: "42 lines".to_string(),
            },
        );
        assert!(updated);
        match &m.parts[0] {
            ChatPart::Tool { state, .. } => {
                assert!(matches!(state, ToolPartState::Completed { .. }));
                assert!(state.is_terminal());
            }
            _ => panic!("expected Tool part"),
        }
    }

    #[test]
    fn complete_tool_returns_false_for_unknown_call_id() {
        let mut m = ChatMessage::new(MessageRole::Assistant, String::new());
        m.push_tool(
            "read".to_string(),
            "call_3".to_string(),
            ToolPartState::Pending { input: serde_json::Value::Null },
        );
        let updated = m.complete_tool(
            "nonexistent",
            ToolPartState::Completed {
                input: serde_json::Value::Null,
                output: String::new(),
            },
        );
        assert!(!updated);
    }

    #[test]
    fn complete_tool_skips_already_terminal_parts() {
        let mut m = ChatMessage::new(MessageRole::Assistant, String::new());
        m.push_tool(
            "read".to_string(),
            "call_4".to_string(),
            ToolPartState::Completed {
                input: serde_json::Value::Null,
                output: "done".to_string(),
            },
        );
        let updated = m.complete_tool(
            "call_4",
            ToolPartState::Error {
                input: serde_json::Value::Null,
                error: "boom".to_string(),
            },
        );
        assert!(!updated);
    }

    #[test]
    fn input_preview_handles_objects_and_long_values() {
        let short = input_preview(&serde_json::json!({"a": "b"}));
        assert!(short.contains("a=b"));
        let long_val = "x".repeat(100);
        let prev = input_preview(&serde_json::json!({"k": long_val}));
        // Should be truncated with ellipsis.
        assert!(prev.contains("…"));
        assert!(prev.len() < 60);
        assert_eq!(input_preview(&serde_json::Value::Null), "");
    }

    #[test]
    fn refresh_message_text_rebuilds_from_parts() {
        let mut m = ChatMessage::new(MessageRole::Assistant, "old text".to_string());
        m.parts.clear();
        m.push_text("hello");
        m.push_tool(
            "bash".to_string(),
            "c1".to_string(),
            ToolPartState::Pending { input: serde_json::json!({"cmd": "ls"}) },
        );
        refresh_message_text(&mut m);
        assert!(m.text.contains("hello"));
        assert!(m.text.contains("bash"));
        assert!(m.text.contains("ls"));
    }
