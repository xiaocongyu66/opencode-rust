use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LocationData {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent: Option<Vec<serde_json::Value>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command: Option<Vec<serde_json::Value>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub integration: Option<Vec<serde_json::Value>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<Vec<serde_json::Value>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider: Option<Vec<serde_json::Value>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reference: Option<Vec<serde_json::Value>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skill: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SessionData {
    pub info: HashMap<String, serde_json::Value>,
    pub message: HashMap<String, Vec<SessionMessage>>,
    pub permission: HashMap<String, Vec<serde_json::Value>>,
    pub question: HashMap<String, Vec<serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProjectData {
    pub permission: HashMap<String, Vec<serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Data {
    pub session: SessionData,
    pub project: ProjectData,
    pub location: HashMap<String, LocationData>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum SessionMessage {
    #[serde(rename = "user")]
    User {
        id: String,
        text: String,
        #[serde(default)]
        files: Vec<String>,
        #[serde(default)]
        agents: Vec<String>,
        time: MessageTime,
    },
    #[serde(rename = "assistant")]
    Assistant {
        id: String,
        agent: serde_json::Value,
        model: serde_json::Value,
        content: Vec<serde_json::Value>,
        time: MessageTime,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        finish: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        error: Option<String>,
    },
    #[serde(rename = "shell")]
    Shell {
        id: String,
        call_id: String,
        command: String,
        output: String,
        time: MessageTime,
    },
    #[serde(rename = "system")]
    System {
        id: String,
        text: String,
        time: MessageTime,
    },
    #[serde(rename = "synthetic")]
    Synthetic {
        id: String,
        session_id: String,
        text: String,
        time: MessageTime,
    },
    #[serde(rename = "agent-switched")]
    AgentSwitched {
        id: String,
        agent: serde_json::Value,
        time: MessageTime,
    },
    #[serde(rename = "model-switched")]
    ModelSwitched {
        id: String,
        model: serde_json::Value,
        time: MessageTime,
    },
    #[serde(rename = "compaction")]
    Compaction {
        id: String,
        reason: String,
        summary: String,
        recent: serde_json::Value,
        time: MessageTime,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct MessageTime {
    pub created: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub completed: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ran: Option<f64>,
}

pub fn location_key(directory: &str, workspace_id: Option<&str>) -> String {
    serde_json::to_string(&(directory, workspace_id)).unwrap_or_default()
}

#[derive(Clone)]
pub struct DataContext {
    pub data: Arc<Mutex<Data>>,
    pub default_directory: String,
    pub default_workspace: Option<String>,
}

impl DataContext {
    pub fn new(default_directory: String) -> Self {
        Self {
            data: Arc::new(Mutex::new(Data::default())),
            default_directory,
            default_workspace: None,
        }
    }

    pub fn session_get(&self, session_id: &str) -> Option<serde_json::Value> {
        self.data.lock().unwrap().session.info.get(session_id).cloned()
    }

    pub fn session_message_list(&self, session_id: &str) -> Vec<SessionMessage> {
        self.data
            .lock()
            .unwrap()
            .session
            .message
            .get(session_id)
            .cloned()
            .unwrap_or_default()
    }

    pub fn session_message_update<F>(&self, session_id: &str, f: F)
    where
        F: FnOnce(&mut Vec<SessionMessage>),
    {
        let mut guard = self.data.lock().unwrap();
        let messages = guard
            .session
            .message
            .entry(session_id.to_string())
            .or_default();
        f(messages);
    }

    pub fn session_message_prepend(messages: &mut Vec<SessionMessage>, item: SessionMessage) {
        if messages.iter().any(|m| m.id() == item.id()) {
            return;
        }
        messages.insert(0, item);
    }

    pub fn active_assistant(messages: &[SessionMessage]) -> Option<usize> {
        messages
            .iter()
            .position(|m| matches!(m, SessionMessage::Assistant { time, .. } if time.completed.is_none()))
    }

    pub fn assistant_by_id(messages: &[SessionMessage], message_id: &str) -> Option<usize> {
        messages.iter().position(|m| {
            matches!(m, SessionMessage::Assistant { id, .. } if id == message_id)
        })
    }

    pub fn active_shell(messages: &[SessionMessage], call_id: &str) -> Option<usize> {
        messages
            .iter()
            .position(|m| matches!(m, SessionMessage::Shell { call_id: cid, .. } if cid == call_id))
    }

    pub fn latest_tool(assistant: &SessionMessage, call_id: Option<&str>) -> Option<usize> {
        if let SessionMessage::Assistant { content, .. } = assistant {
            content
                .iter()
                .rposition(|item| {
                    item.get("type").and_then(|v| v.as_str()) == Some("tool")
                        && call_id.map_or(true, |cid| {
                            item.get("id").and_then(|v| v.as_str()) == Some(cid)
                        })
                })
                .map(|_| ())
        } else {
            None
        }
        .map(|_| 0)
    }

    pub fn latest_text(assistant: &SessionMessage, text_id: &str) -> Option<usize> {
        if let SessionMessage::Assistant { content, .. } = assistant {
            content.iter().rposition(|item| {
                item.get("type").and_then(|v| v.as_str()) == Some("text")
                    && item.get("id").and_then(|v| v.as_str()) == Some(text_id)
            })
        } else {
            None
        }
    }

    pub fn latest_reasoning(assistant: &SessionMessage, reasoning_id: &str) -> Option<usize> {
        if let SessionMessage::Assistant { content, .. } = assistant {
            content.iter().rposition(|item| {
                item.get("type").and_then(|v| v.as_str()) == Some("reasoning")
                    && item.get("id").and_then(|v| v.as_str()) == Some(reasoning_id)
            })
        } else {
            None
        }
    }

    pub fn location_default(&self) -> (String, Option<String>) {
        (self.default_directory.clone(), self.default_workspace.clone())
    }

    pub fn location_agent_list(&self, location: Option<(&str, Option<&str>)>) -> Vec<serde_json::Value> {
        let (dir, ws) = location.unwrap_or_else(|| self.location_default_ref());
        let key = location_key(&dir, ws);
        self.data
            .lock()
            .unwrap()
            .location
            .get(&key)
            .and_then(|d| d.agent.clone())
            .unwrap_or_default()
    }

    fn location_default_ref(&self) -> (String, Option<String>) {
        (self.default_directory.clone(), self.default_workspace.clone())
    }

    pub fn location_provider_list(&self, location: Option<(&str, Option<&str>)>) -> Vec<serde_json::Value> {
        let (dir, ws) = location.unwrap_or_else(|| self.location_default_ref());
        let key = location_key(&dir, ws);
        self.data
            .lock()
            .unwrap()
            .location
            .get(&key)
            .and_then(|d| d.provider.clone())
            .unwrap_or_default()
    }

    pub fn location_model_list(&self, location: Option<(&str, Option<&str>)>) -> Vec<serde_json::Value> {
        let (dir, ws) = location.unwrap_or_else(|| self.location_default_ref());
        let key = location_key(&dir, ws);
        self.data
            .lock()
            .unwrap()
            .location
            .get(&key)
            .and_then(|d| d.model.clone())
            .unwrap_or_default()
    }

    pub fn location_set_agents(&self, key: &str, agents: Vec<serde_json::Value>) {
        let mut guard = self.data.lock().unwrap();
        guard.location.entry(key.to_string()).or_default().agent = Some(agents);
    }

    pub fn location_set_providers(&self, key: &str, providers: Vec<serde_json::Value>) {
        let mut guard = self.data.lock().unwrap();
        guard.location.entry(key.to_string()).or_default().provider = Some(providers);
    }

    pub fn location_set_models(&self, key: &str, models: Vec<serde_json::Value>) {
        let mut guard = self.data.lock().unwrap();
        guard.location.entry(key.to_string()).or_default().model = Some(models);
    }

    pub fn location_set_commands(&self, key: &str, commands: Vec<serde_json::Value>) {
        let mut guard = self.data.lock().unwrap();
        guard.location.entry(key.to_string()).or_default().command = Some(commands);
    }

    pub fn location_set_skills(&self, key: &str, skills: Vec<serde_json::Value>) {
        let mut guard = self.data.lock().unwrap();
        guard.location.entry(key.to_string()).or_default().skill = Some(skills);
    }

    pub fn location_set_integrations(&self, key: &str, integrations: Vec<serde_json::Value>) {
        let mut guard = self.data.lock().unwrap();
        guard.location.entry(key.to_string()).or_default().integration = Some(integrations);
    }

    pub fn location_set_references(&self, key: &str, references: Vec<serde_json::Value>) {
        let mut guard = self.data.lock().unwrap();
        guard.location.entry(key.to_string()).or_default().reference = Some(references);
    }
}

impl SessionMessage {
    pub fn id(&self) -> &str {
        match self {
            SessionMessage::User { id, .. } => id,
            SessionMessage::Assistant { id, .. } => id,
            SessionMessage::Shell { id, .. } => id,
            SessionMessage::System { id, .. } => id,
            SessionMessage::Synthetic { id, .. } => id,
            SessionMessage::AgentSwitched { id, .. } => id,
            SessionMessage::ModelSwitched { id, .. } => id,
            SessionMessage::Compaction { id, .. } => id,
        }
    }
}
