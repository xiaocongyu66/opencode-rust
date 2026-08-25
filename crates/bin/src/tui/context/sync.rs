use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VcsInfo {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConsoleState {
    #[serde(default)]
    pub console_managed_providers: Vec<String>,
    #[serde(default)]
    pub switchable_org_count: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Capabilities {
    pub experimental_background_subagents: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProviderListResponse {
    #[serde(default)]
    pub all: Vec<serde_json::Value>,
    #[serde(default)]
    pub default: HashMap<String, String>,
    #[serde(default)]
    pub connected: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SyncStatus {
    Loading,
    Partial,
    Complete,
}

pub struct SyncStore {
    pub status: SyncStatus,
    pub provider: Vec<serde_json::Value>,
    pub provider_default: HashMap<String, String>,
    pub provider_next: ProviderListResponse,
    pub console_state: ConsoleState,
    pub capabilities: Capabilities,
    pub provider_auth: HashMap<String, Vec<serde_json::Value>>,
    pub agent: Vec<serde_json::Value>,
    pub command: Vec<serde_json::Value>,
    pub permission: HashMap<String, Vec<serde_json::Value>>,
    pub question: HashMap<String, Vec<serde_json::Value>>,
    pub config: serde_json::Value,
    pub session: Vec<serde_json::Value>,
    pub session_status: HashMap<String, String>,
    pub session_diff: HashMap<String, Vec<serde_json::Value>>,
    pub todo: HashMap<String, Vec<serde_json::Value>>,
    pub message: HashMap<String, Vec<serde_json::Value>>,
    pub part: HashMap<String, Vec<serde_json::Value>>,
    pub lsp: Vec<serde_json::Value>,
    pub mcp: HashMap<String, serde_json::Value>,
    pub mcp_resource: HashMap<String, serde_json::Value>,
    pub formatter: Vec<serde_json::Value>,
    pub vcs: Option<VcsInfo>,
}

impl Default for SyncStore {
    fn default() -> Self {
        Self {
            status: SyncStatus::Loading,
            provider: Vec::new(),
            provider_default: HashMap::new(),
            provider_next: ProviderListResponse::default(),
            console_state: ConsoleState::default(),
            capabilities: Capabilities::default(),
            provider_auth: HashMap::new(),
            agent: Vec::new(),
            command: Vec::new(),
            permission: HashMap::new(),
            question: HashMap::new(),
            config: serde_json::Value::Object(serde_json::Map::new()),
            session: Vec::new(),
            session_status: HashMap::new(),
            session_diff: HashMap::new(),
            todo: HashMap::new(),
            message: HashMap::new(),
            part: HashMap::new(),
            lsp: Vec::new(),
            mcp: HashMap::new(),
            mcp_resource: HashMap::new(),
            formatter: Vec::new(),
            vcs: None,
        }
    }
}

pub struct SyncContext {
    pub data: Arc<Mutex<SyncStore>>,
    pub skip_initial_loading: bool,
    full_synced_sessions: Arc<Mutex<HashSet<String>>>,
}

use std::collections::HashSet;

impl SyncContext {
    pub fn new(skip_initial_loading: bool) -> Self {
        Self {
            data: Arc::new(Mutex::new(SyncStore::default())),
            skip_initial_loading,
            full_synced_sessions: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    pub fn status(&self) -> SyncStatus {
        self.data.lock().unwrap().status.clone()
    }

    pub fn set_status(&self, status: SyncStatus) {
        self.data.lock().unwrap().status = status;
    }

    pub fn ready(&self) -> bool {
        if self.skip_initial_loading {
            return true;
        }
        self.status() != SyncStatus::Loading
    }

    pub fn vcs_branch(&self) -> Option<String> {
        self.data
            .lock()
            .unwrap()
            .vcs
            .as_ref()
            .and_then(|v| v.branch.clone())
    }

    pub fn session_get(&self, session_id: &str) -> Option<serde_json::Value> {
        let store = self.data.lock().unwrap();
        store
            .session
            .iter()
            .find(|s| s.get("id").and_then(|v| v.as_str()) == Some(session_id))
            .cloned()
    }

    pub fn session_status(&self, session_id: &str) -> String {
        let store = self.data.lock().unwrap();
        if let Some(status) = store.session_status.get(session_id) {
            return status.clone();
        }
        let session = store
            .session
            .iter()
            .find(|s| s.get("id").and_then(|v| v.as_str()) == Some(session_id));
        let session = match session {
            None => return "idle".to_string(),
            Some(s) => s,
        };
        if session
            .get("time")
            .and_then(|t| t.get("compacting"))
            .is_some()
        {
            return "compacting".to_string();
        }
        let messages = store.message.get(session_id);
        match messages.and_then(|m| m.last()) {
            None => "idle".to_string(),
            Some(last) => {
                let role = last.get("role").and_then(|v| v.as_str()).unwrap_or("");
                if role == "user" {
                    "working".to_string()
                } else {
                    let completed = last
                        .get("time")
                        .and_then(|t| t.get("completed"))
                        .is_some();
                    if completed { "idle" } else { "working" }.to_string()
                }
            }
        }
    }

    pub fn session_fully_synced(&self, session_id: &str) -> bool {
        self.full_synced_sessions
            .lock()
            .unwrap()
            .contains(session_id)
    }

    pub fn mark_session_synced(&self, session_id: &str) {
        self.full_synced_sessions
            .lock()
            .unwrap()
            .insert(session_id.to_string());
    }

    pub fn set_providers(&self, providers: Vec<serde_json::Value>, defaults: HashMap<String, String>) {
        let mut store = self.data.lock().unwrap();
        store.provider = providers;
        store.provider_default = defaults;
    }

    pub fn set_provider_next(&self, response: ProviderListResponse) {
        self.data.lock().unwrap().provider_next = response;
    }

    pub fn set_agents(&self, agents: Vec<serde_json::Value>) {
        self.data.lock().unwrap().agent = agents;
    }

    pub fn set_config(&self, config: serde_json::Value) {
        self.data.lock().unwrap().config = config;
    }

    pub fn set_sessions(&self, sessions: Vec<serde_json::Value>) {
        self.data.lock().unwrap().session = sessions;
    }

    pub fn set_commands(&self, commands: Vec<serde_json::Value>) {
        self.data.lock().unwrap().command = commands;
    }

    pub fn set_lsp(&self, lsp: Vec<serde_json::Value>) {
        self.data.lock().unwrap().lsp = lsp;
    }

    pub fn set_mcp(&self, mcp: HashMap<String, serde_json::Value>) {
        self.data.lock().unwrap().mcp = mcp;
    }

    pub fn set_mcp_resource(&self, resources: HashMap<String, serde_json::Value>) {
        self.data.lock().unwrap().mcp_resource = resources;
    }

    pub fn set_formatter(&self, formatter: Vec<serde_json::Value>) {
        self.data.lock().unwrap().formatter = formatter;
    }

    pub fn set_session_status_map(&self, status: HashMap<String, String>) {
        self.data.lock().unwrap().session_status = status;
    }

    pub fn set_provider_auth(&self, auth: HashMap<String, Vec<serde_json::Value>>) {
        self.data.lock().unwrap().provider_auth = auth;
    }

    pub fn set_vcs(&self, vcs: Option<VcsInfo>) {
        self.data.lock().unwrap().vcs = vcs;
    }

    pub fn set_console_state(&self, state: ConsoleState) {
        self.data.lock().unwrap().console_state = state;
    }

    pub fn set_capabilities(&self, caps: Capabilities) {
        self.data.lock().unwrap().capabilities = caps;
    }

    pub fn add_permission(&self, session_id: &str, request: serde_json::Value) {
        self.data
            .lock()
            .unwrap()
            .permission
            .entry(session_id.to_string())
            .or_default()
            .push(request);
    }

    pub fn remove_permission(&self, session_id: &str, request_id: &str) {
        let mut store = self.data.lock().unwrap();
        if let Some(requests) = store.permission.get_mut(session_id) {
            requests.retain(|r| r.get("id").and_then(|v| v.as_str()) != Some(request_id));
        }
    }

    pub fn add_question(&self, session_id: &str, request: serde_json::Value) {
        self.data
            .lock()
            .unwrap()
            .question
            .entry(session_id.to_string())
            .or_default()
            .push(request);
    }

    pub fn remove_question(&self, session_id: &str, request_id: &str) {
        let mut store = self.data.lock().unwrap();
        if let Some(requests) = store.question.get_mut(session_id) {
            requests.retain(|r| r.get("id").and_then(|v| v.as_str()) != Some(request_id));
        }
    }

    pub fn set_todos(&self, session_id: &str, todos: Vec<serde_json::Value>) {
        self.data.lock().unwrap().todo.insert(session_id.to_string(), todos);
    }

    pub fn set_session_diff(&self, session_id: &str, diff: Vec<serde_json::Value>) {
        self.data.lock().unwrap().session_diff.insert(session_id.to_string(), diff);
    }

    pub fn upsert_session(&self, session: serde_json::Value) {
        let id = session.get("id").and_then(|v| v.as_str()).map(|s| s.to_string());
        let id = match id {
            Some(id) => id,
            None => return,
        };
        let mut store = self.data.lock().unwrap();
        if let Some(pos) = store.session.iter().position(|s| {
            s.get("id").and_then(|v| v.as_str()) == Some(&id)
        }) {
            store.session[pos] = session;
        } else {
            store.session.push(session);
        }
    }

    pub fn remove_session(&self, session_id: &str) {
        let mut store = self.data.lock().unwrap();
        store.session.retain(|s| s.get("id").and_then(|v| v.as_str()) != Some(session_id));
    }

    pub fn set_session_status_for(&self, session_id: &str, status: &str) {
        self.data.lock().unwrap().session_status.insert(session_id.to_string(), status.to_string());
    }

    pub fn upsert_message(&self, session_id: &str, message: serde_json::Value) {
        let msg_id = message.get("id").and_then(|v| v.as_str()).map(|s| s.to_string());
        let msg_id = match msg_id {
            Some(id) => id,
            None => return,
        };
        let created = message
            .get("time")
            .and_then(|t| t.get("created"))
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        let key = format!("{}{}", created, msg_id);

        let mut store = self.data.lock().unwrap();
        let messages = store.message.entry(session_id.to_string()).or_default();

        let pos = messages.iter().position(|m| {
            let m_created = m.get("time").and_then(|t| t.get("created")).and_then(|v| v.as_f64()).unwrap_or(0.0);
            let m_id = m.get("id").and_then(|v| v.as_str()).unwrap_or("");
            let m_key = format!("{}{}", m_created, m_id);
            m_key == key
        });

        match pos {
            Some(idx) => messages[idx] = message,
            None => {
                let insert_pos = messages.partition_point(|m| {
                    let m_created = m.get("time").and_then(|t| t.get("created")).and_then(|v| v.as_f64()).unwrap_or(0.0);
                    let m_id = m.get("id").and_then(|v| v.as_str()).unwrap_or("");
                    let m_key = format!("{}{}", m_created, m_id);
                    m_key < key
                });
                messages.insert(insert_pos, message);

                if messages.len() > 100 {
                    let removed = messages.remove(0);
                    if let Some(removed_id) = removed.get("id").and_then(|v| v.as_str()).map(|s| s.to_string()) {
                        store.part.remove(&removed_id);
                    }
                }
            }
        }
    }

    pub fn remove_message(&self, session_id: &str, message_id: &str) {
        let mut store = self.data.lock().unwrap();
        if let Some(messages) = store.message.get_mut(session_id) {
            messages.retain(|m| m.get("id").and_then(|v| v.as_str()) != Some(message_id));
        }
    }

    pub fn upsert_part(&self, message_id: &str, part: serde_json::Value) {
        let part_id = part.get("id").and_then(|v| v.as_str()).map(|s| s.to_string());
        let part_id = match part_id {
            Some(id) => id,
            None => return,
        };
        let mut store = self.data.lock().unwrap();
        let parts = store.part.entry(message_id.to_string()).or_default();
        if let Some(pos) = parts.iter().position(|p| p.get("id").and_then(|v| v.as_str()) == Some(&part_id)) {
            parts[pos] = part;
        } else {
            parts.push(part);
        }
    }

    pub fn append_part_delta(&self, message_id: &str, part_id: &str, field: &str, delta: &str) {
        let mut store = self.data.lock().unwrap();
        if let Some(parts) = store.part.get_mut(message_id) {
            if let Some(part) = parts.iter_mut().find(|p| p.get("id").and_then(|v| v.as_str()) == Some(part_id)) {
                if let Some(existing) = part.get(field).and_then(|v| v.as_str()) {
                    let new_val = format!("{}{}", existing, delta);
                    if let Some(obj) = part.as_object_mut() {
                        obj.insert(field.to_string(), serde_json::Value::String(new_val));
                    }
                } else {
                    if let Some(obj) = part.as_object_mut() {
                        obj.insert(field.to_string(), serde_json::Value::String(delta.to_string()));
                    }
                }
            }
        }
    }

    pub fn remove_part(&self, message_id: &str, part_id: &str) {
        let mut store = self.data.lock().unwrap();
        if let Some(parts) = store.part.get_mut(message_id) {
            parts.retain(|p| p.get("id").and_then(|v| v.as_str()) != Some(part_id));
        }
    }
}

pub fn binary_search<T, F>(items: &[T], target: &str, key: F) -> (bool, usize)
where
    F: Fn(&T) -> &str,
{
    let mut left = 0;
    let mut right = items.len().saturating_sub(1);
    while left <= right {
        let mid = (left + right) / 2;
        let value = key(&items[mid]);
        if value == target {
            return (true, mid);
        }
        if value < target {
            left = mid + 1;
        } else {
            if mid == 0 {
                break;
            }
            right = mid - 1;
        }
    }
    (false, left)
}

pub fn compare_message(a: &serde_json::Value, b: &serde_json::Value) -> std::cmp::Ordering {
    let a_created = a.get("time").and_then(|t| t.get("created")).and_then(|v| v.as_f64()).unwrap_or(0.0);
    let b_created = b.get("time").and_then(|t| t.get("created")).and_then(|v| v.as_f64()).unwrap_or(0.0);
    let a_id = a.get("id").and_then(|v| v.as_str()).unwrap_or("");
    let b_id = b.get("id").and_then(|v| v.as_str()).unwrap_or("");
    a_created.partial_cmp(&b_created)
        .unwrap_or(std::cmp::Ordering::Equal)
        .then_with(|| a_id.cmp(b_id))
}

pub fn message_key(message: &serde_json::Value) -> String {
    let created = message.get("time").and_then(|t| t.get("created")).and_then(|v| v.as_f64()).unwrap_or(0.0);
    let id = message.get("id").and_then(|v| v.as_str()).unwrap_or("");
    format!("{}{}", created, id)
}
