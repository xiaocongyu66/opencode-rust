use crate::tui::plugins::builtins::BuiltinTuiPlugin;
use std::collections::HashSet;

pub const NOTIFICATIONS_ID: &str = "internal:notifications";

pub struct NotificationsPlugin;

impl NotificationsPlugin {
    pub fn builtin() -> BuiltinTuiPlugin {
        BuiltinTuiPlugin::new(NOTIFICATIONS_ID).with_order(100)
    }

    pub fn id() -> &'static str { NOTIFICATIONS_ID }
}

pub type AttentionSoundName = &'static str;
pub const SOUND_QUESTION: AttentionSoundName = "question";
pub const SOUND_PERMISSION: AttentionSoundName = "permission";
pub const SOUND_DONE: AttentionSoundName = "done";
pub const SOUND_SUBAGENT_DONE: AttentionSoundName = "subagent_done";
pub const SOUND_ERROR: AttentionSoundName = "error";

pub struct NotifyOptions {
    pub title: Option<String>,
    pub message: String,
    pub notification_when: &'static str,
    pub sound: AttentionSoundName,
    pub sound_when: &'static str,
}

pub fn notify(session_title: Option<&str>, message: &str, sound: AttentionSoundName, is_subagent: bool) -> NotifyOptions {
    NotifyOptions {
        title: session_title.map(|s| s.to_string()),
        message: message.to_string(),
        notification_when: if is_subagent { "never" } else { "blurred" },
        sound,
        sound_when: "always",
    }
}

pub fn session_error_message(error_name: Option<&str>, error_data_message: Option<&str>) -> &'static str {
    if error_name == Some("MessageAbortedError") {
        return "Session aborted";
    }
    if let Some(msg) = error_data_message {
        if msg == "SSE read timed out" {
            return "Model stopped responding";
        }
    }
    "Session error"
}

pub struct NotificationState {
    pub active: HashSet<String>,
    pub errored: HashSet<String>,
    pub questions: HashSet<String>,
    pub permissions: HashSet<String>,
}

impl NotificationState {
    pub fn new() -> Self {
        Self {
            active: HashSet::new(),
            errored: HashSet::new(),
            questions: HashSet::new(),
            permissions: HashSet::new(),
        }
    }

    pub fn on_question_asked(&mut self, id: &str, session_id: Option<&str>, session_title: Option<&str>, is_subagent: bool) -> Option<NotifyOptions> {
        if self.questions.contains(id) { return None }
        self.questions.insert(id.to_string());
        Some(notify(session_title, "Question needs input", SOUND_QUESTION, is_subagent))
    }

    pub fn on_question_replied(&mut self, request_id: &str) {
        self.questions.remove(request_id);
    }

    pub fn on_question_rejected(&mut self, request_id: &str) {
        self.questions.remove(request_id);
    }

    pub fn on_permission_asked(&mut self, id: &str, session_title: Option<&str>, is_subagent: bool) -> Option<NotifyOptions> {
        if self.permissions.contains(id) { return None }
        self.permissions.insert(id.to_string());
        Some(notify(session_title, "Permission needs input", SOUND_PERMISSION, is_subagent))
    }

    pub fn on_permission_replied(&mut self, request_id: &str) {
        self.permissions.remove(request_id);
    }

    pub fn on_session_status_busy(&mut self, session_id: &str) {
        self.active.insert(session_id.to_string());
        self.errored.remove(session_id);
    }

    pub fn on_session_status_idle(&mut self, session_id: &str, session_title: Option<&str>, is_subagent: bool) -> Option<NotifyOptions> {
        if !self.active.contains(session_id) { return None }
        self.active.remove(session_id);

        if self.errored.contains(session_id) {
            self.errored.remove(session_id);
            return None;
        }

        let sound = if is_subagent { SOUND_SUBAGENT_DONE } else { SOUND_DONE };
        Some(notify(session_title, "Session done", sound, is_subagent))
    }

    pub fn on_session_error(&mut self, session_id: &str, error_name: Option<&str>, error_data_message: Option<&str>, session_title: Option<&str>, is_subagent: bool) -> Option<NotifyOptions> {
        if !self.active.contains(session_id) { return None }
        self.errored.insert(session_id.to_string());
        let message = session_error_message(error_name, error_data_message);
        Some(notify(session_title, message, SOUND_ERROR, is_subagent))
    }
}

impl Default for NotificationState {
    fn default() -> Self { Self::new() }
}
