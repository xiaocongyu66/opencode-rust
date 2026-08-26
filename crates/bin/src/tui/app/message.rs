//! Message types for the TUI chat view.
//!
//! `MessageRole`, `ChatPart`, `ChatMessage`, and `ToolPartState` mirror the
//! structured `Part` union in `schema::session::AssistantContent` but are
//! flattened for TUI rendering. Kept in a separate module so `app.rs`
//! stays focused on App state and event handling.

#[derive(Clone, PartialEq, Debug, serde::Serialize, serde::Deserialize)]
pub enum MessageRole {
    User,
    Assistant,
    System,
}

/// State of a tool call part — mirrors `schema::session::ToolState` but
/// simplified for TUI display (no time/attachments/structured metadata).
#[derive(Clone, Debug)]
pub enum ToolPartState {
    /// Tool call was announced by the LLM but hasn't completed yet.
    Pending {
        input: serde_json::Value,
    },
    /// Tool completed successfully.
    Completed {
        input: serde_json::Value,
        output: String,
    },
    /// Tool failed.
    Error {
        input: serde_json::Value,
        error: String,
    },
}

impl ToolPartState {
    pub fn status_label(&self) -> &'static str {
        match self {
            ToolPartState::Pending { .. } => "pending",
            ToolPartState::Completed { .. } => "completed",
            ToolPartState::Error { .. } => "error",
        }
    }

    pub fn input(&self) -> &serde_json::Value {
        match self {
            ToolPartState::Pending { input }
            | ToolPartState::Completed { input, .. }
            | ToolPartState::Error { input, .. } => input,
        }
    }

    pub fn is_terminal(&self) -> bool {
        !matches!(self, ToolPartState::Pending { .. })
    }
}

/// A structured piece of a chat message — mirrors the `Part` union in
/// `schema::session::AssistantContent` but flattened for TUI rendering.
#[derive(Clone, Debug)]
pub enum ChatPart {
    /// Plain text (user input or assistant prose).
    Text { text: String },
    /// A tool call with its current state.
    Tool {
        tool_name: String,
        call_id: String,
        state: ToolPartState,
    },
}

impl ChatPart {
    pub fn as_text(&self) -> Option<&str> {
        match self {
            ChatPart::Text { text } => Some(text),
            ChatPart::Tool { .. } => None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ChatMessage {
    pub role: MessageRole,
    pub text: String,
    /// Structured parts (tool calls + text segments). Built up by
    /// `poll_runner_events`. Empty for legacy messages created via
    /// `ChatMessage::new`.
    pub parts: Vec<ChatPart>,
    /// True if this message is queued (sent while the LLM was still
    /// generating). The message is shown immediately with a `QUEUED`
    /// tag, and promoted to active when the LLM finishes the previous
    /// turn. Mirrors the TS `queued` / `pending` index logic.
    pub queued: bool,
}

impl ChatMessage {
    pub fn new(role: MessageRole, text: impl Into<String>) -> Self {
        Self {
            role,
            text: text.into(),
            parts: Vec::new(),
            queued: false,
        }
    }

    /// Append text. If the last part is already a Text part, the text is
    /// appended to it (so streaming chunks merge into one paragraph).
    /// Otherwise a new Text part is started.
    pub fn push_text(&mut self, text: impl Into<String>) {
        let s = text.into();
        self.text.push_str(&s);
        // Try to merge into the last Text part.
        let merge = match self.parts.last_mut() {
            Some(ChatPart::Text { text: existing }) => {
                existing.push_str(&s);
                true
            }
            _ => false,
        };
        if !merge {
            self.parts.push(ChatPart::Text { text: s });
        }
    }

    /// Append a tool part. Does not touch `text` — tool rendering is
    /// part-based. A friendly one-line summary is also pushed to `text`
    /// so legacy text-only renderers still show something.
    pub fn push_tool(&mut self, tool_name: String, call_id: String, state: ToolPartState) {
        let summary = tool_part_summary(&tool_name, &state);
        self.parts.push(ChatPart::Tool {
            tool_name,
            call_id,
            state,
        });
        if !self.text.is_empty() {
            self.text.push('\n');
        }
        self.text.push_str(&summary);
    }

    /// Find a pending tool part with the given call_id and update its state.
    /// Returns true if a matching part was found.
    pub fn complete_tool(
        &mut self,
        call_id: &str,
        new_state: ToolPartState,
    ) -> bool {
        for part in &mut self.parts {
            if let ChatPart::Tool {
                call_id: cid,
                state,
                ..
            } = part
            {
                if cid == call_id && !state.is_terminal() {
                    *state = new_state;
                    return true;
                }
            }
        }
        false
    }
}

/// One-line summary of a tool part, used for the legacy `text` field.
pub fn tool_part_summary(tool_name: &str, state: &ToolPartState) -> String {
    match state {
        ToolPartState::Pending { input } => {
            crate::t!("tui.message.tool", name = tool_name) + " " + &input_preview(input)
        }
        ToolPartState::Completed { output, .. } => {
            let trimmed = output.trim();
            if trimmed.is_empty() {
                crate::t!("tui.message.tool_completed", id = tool_name)
            } else {
                crate::t!("tui.message.tool_result", summary = trimmed)
            }
        }
        ToolPartState::Error { error, .. } => {
            crate::t!("tui.message.tool_failed", id = tool_name, error = error)
        }
    }
}

/// Short preview of a tool input JSON for one-line summaries.
pub fn input_preview(input: &serde_json::Value) -> String {
    match input {
        serde_json::Value::Null => String::new(),
        serde_json::Value::Object(map) => {
            let mut parts = Vec::with_capacity(map.len());
            for (k, v) in map {
                let vstr = match v {
                    serde_json::Value::String(s) => s.clone(),
                    other => other.to_string(),
                };
                let vstr = truncate_chars(&vstr, 40);
                parts.push(format!("{k}={vstr}"));
            }
            parts.join(" ")
        }
        other => {
            let s = other.to_string();
            truncate_chars(&s, 60)
        }
    }
}

/// Truncate a string to at most `max` **characters** (not bytes),
/// appending an ellipsis if truncated. Safe for multi-byte UTF-8.
pub fn truncate_chars(s: &str, max: usize) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() <= max {
        return s.to_string();
    }
    let head: String = chars.into_iter().take(max).collect();
    format!("{head}…")
}

/// Rebuild the legacy `text` field of a message from its structured parts.
/// Called after a tool part's state changes so text-only renderers stay
/// consistent with the part-based view.
pub fn refresh_message_text(msg: &mut ChatMessage) {
    let mut buf = String::new();
    for (i, part) in msg.parts.iter().enumerate() {
        if i > 0 {
            buf.push('\n');
        }
        match part {
            ChatPart::Text { text } => buf.push_str(text),
            ChatPart::Tool {
                tool_name,
                state,
                ..
            } => buf.push_str(&tool_part_summary(tool_name, state)),
        }
    }
    msg.text = buf;
}

#[cfg(test)]
mod tests {
    use super::*;

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
        // Consecutive push_text calls merge into a single Text part so
        // streaming chunks don't create one paragraph per chunk.
        assert_eq!(m.text, "hello world");
        assert_eq!(m.parts.len(), 1);
        assert_eq!(m.parts[0].as_text(), Some("hello world"));
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
}
