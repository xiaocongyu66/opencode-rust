use std::sync::{Arc, Mutex};

pub struct EditorContext {
    inner: Arc<Mutex<EditorInner>>,
}

#[derive(Debug, Clone, Default)]
pub struct EditorPosition {
    pub line: i64,
    pub character: i64,
}

#[derive(Debug, Clone, Default)]
pub struct EditorSelectionRange {
    pub text: String,
    pub start: EditorPosition,
    pub end: EditorPosition,
}

#[derive(Debug, Clone, Default)]
pub struct EditorSelection {
    pub file_path: String,
    pub source: Option<String>,
    pub ranges: Vec<EditorSelectionRange>,
}

#[derive(Debug, Clone, Default)]
pub struct EditorServerInfo {
    pub protocol_version: Option<String>,
    pub name: Option<String>,
    pub version: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EditorLabelState {
    Pending,
    Sent,
    None,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EditorStatus {
    Disabled,
    Connecting,
    Connected,
}

struct EditorInner {
    status: EditorStatus,
    selection: Option<EditorSelection>,
    selection_sent: bool,
    server: Option<EditorServerInfo>,
    directory: String,
    port: Option<u16>,
    zed_terminal: bool,
    closed: bool,
}

impl Default for EditorInner {
    fn default() -> Self {
        Self {
            status: EditorStatus::Disabled,
            selection: None,
            selection_sent: false,
            server: None,
            directory: String::new(),
            port: None,
            zed_terminal: false,
            closed: false,
        }
    }
}

impl EditorContext {
    pub fn new(directory: String) -> Self {
        let port = std::env::var("OPENCODE_EDITOR_SSE_PORT")
            .or_else(|_| std::env::var("CLAUDE_CODE_SSE_PORT"))
            .ok()
            .and_then(|v| v.parse::<u16>().ok())
            .filter(|&p| p > 0);
        let zed_terminal = std::env::var("ZED_TERM").map_or(false, |v| v == "true")
            || std::env::var("TERM_PROGRAM").map_or(false, |v| v.to_lowercase() == "zed");
        Self {
            inner: Arc::new(Mutex::new(EditorInner {
                directory,
                port,
                zed_terminal,
                ..Default::default()
            })),
        }
    }

    pub fn enabled(&self) -> bool {
        let inner = self.inner.lock().unwrap();
        inner.port.is_some() || (inner.zed_terminal)
    }

    pub fn connected(&self) -> bool {
        self.inner.lock().unwrap().status == EditorStatus::Connected
    }

    pub fn selection(&self) -> Option<EditorSelection> {
        self.inner.lock().unwrap().selection.clone()
    }

    pub fn clear_selection(&self) {
        let mut inner = self.inner.lock().unwrap();
        inner.selection = None;
        inner.selection_sent = false;
    }

    pub fn mark_selection_sent(&self) {
        let mut inner = self.inner.lock().unwrap();
        if inner.selection.is_some() {
            inner.selection_sent = true;
        }
    }

    pub fn label_state(&self) -> EditorLabelState {
        let inner = self.inner.lock().unwrap();
        if inner.selection.is_none() {
            return EditorLabelState::None;
        }
        if inner.selection_sent {
            EditorLabelState::Sent
        } else {
            EditorLabelState::Pending
        }
    }

    pub fn server(&self) -> Option<EditorServerInfo> {
        self.inner.lock().unwrap().server.clone()
    }

    pub fn set_status(&self, status: EditorStatus) {
        self.inner.lock().unwrap().status = status;
    }

    pub fn set_selection(&self, selection: EditorSelection) {
        let mut inner = self.inner.lock().unwrap();
        let changed = editor_selection_key(&inner.selection) != editor_selection_key(&Some(selection.clone()));
        inner.selection = Some(selection);
        if changed {
            inner.selection_sent = false;
        }
    }

    pub fn set_server(&self, server: EditorServerInfo) {
        self.inner.lock().unwrap().server = Some(server);
    }

    pub fn directory(&self) -> String {
        self.inner.lock().unwrap().directory.clone()
    }

    pub fn set_directory(&self, dir: &str) {
        self.inner.lock().unwrap().directory = dir.to_string();
    }

    pub fn reconnect(&self, directory: Option<&str>) {
        let resolved = directory.unwrap_or(&self.directory()).to_string();
        self.set_directory(&resolved);
        self.clear_selection();
        self.set_status(EditorStatus::Disabled);
    }
}

pub fn editor_selection_key(selection: &Option<EditorSelection>) -> String {
    match selection {
        None => String::new(),
        Some(sel) => {
            let mut parts = vec![sel.file_path.clone()];
            for range in &sel.ranges {
                parts.push(range.start.line.to_string());
                parts.push(range.start.character.to_string());
                parts.push(range.end.line.to_string());
                parts.push(range.end.character.to_string());
                parts.push(range.text.clone());
            }
            parts.join("\0")
        }
    }
}
