use super::kv::KvContext;
use super::permission::PermissionContext;
use super::sync::SyncContext;
use super::runtime::TuiPaths;
use super::args::Args;
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ModelSelection {
    pub provider_id: String,
    pub model_id: String,
}

pub fn parse_model(model: &str) -> ModelSelection {
    let parts: Vec<&str> = model.splitn(2, '/').collect();
    if parts.len() >= 2 {
        ModelSelection {
            provider_id: parts[0].to_string(),
            model_id: parts[1..].join("/"),
        }
    } else {
        ModelSelection {
            provider_id: parts[0].to_string(),
            model_id: String::new(),
        }
    }
}

pub fn recent_models(
    model: &ModelSelection,
    recent: &[ModelSelection],
) -> Vec<ModelSelection> {
    let mut seen = HashSet::new();
    let mut result = Vec::new();
    let key = format!("{}/{}", model.provider_id, model.model_id);
    seen.insert(key);
    result.push(model.clone());
    for item in recent {
        let k = format!("{}/{}", item.provider_id, item.model_id);
        if !seen.contains(&k) {
            seen.insert(k);
            result.push(item.clone());
        }
        if result.len() >= 10 {
            break;
        }
    }
    result
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ModelStore {
    pub ready: bool,
    pub model: std::collections::HashMap<String, ModelSelection>,
    pub recent: Vec<ModelSelection>,
    pub favorite: Vec<ModelSelection>,
    pub variant: std::collections::HashMap<String, String>,
}

pub struct LocalContext {
    pub model: Arc<Mutex<ModelStore>>,
    pub model_file: PathBuf,
    pub pinned: Arc<Mutex<Vec<String>>>,
    pub session_file: PathBuf,
    pub sync: Arc<SyncContext>,
    pub args: Args,
}

impl LocalContext {
    pub fn new(sync: Arc<SyncContext>, paths: &TuiPaths, args: Args) -> Self {
        let model_file = PathBuf::from(&paths.state).join("model.json");
        let session_file = PathBuf::from(&paths.state).join("session.json");

        let model_store = load_json::<ModelStore>(&model_file).unwrap_or_default();
        let session_store: SessionFile = load_json(&session_file).unwrap_or_default();

        Self {
            model: Arc::new(Mutex::new(model_store)),
            model_file,
            pinned: Arc::new(Mutex::new(session_store.pinned)),
            session_file,
            sync,
            args,
        }
    }

    pub fn is_model_valid(&self, model: &ModelSelection) -> bool {
        let store = self.sync.data.lock().unwrap();
        store.provider.iter().any(|p| {
            p.get("id").and_then(|v| v.as_str()) == Some(&model.provider_id)
                && p.get("models")
                    .and_then(|m| m.as_object())
                    .map(|m| m.contains_key(&model.model_id))
                    .unwrap_or(false)
        })
    }

    pub fn current_model(&self) -> Option<ModelSelection> {
        let store = self.model.lock().unwrap();
        if !store.model.is_empty() {
            if let Some(agent) = self.current_agent_name() {
                if let Some(m) = store.model.get(&agent) {
                    return Some(m.clone());
                }
            }
        }
        self.fallback_model()
    }

    fn current_agent_name(&self) -> Option<String> {
        let store = self.sync.data.lock().unwrap();
        store
            .agent
            .iter()
            .find(|a| {
                a.get("mode").and_then(|v| v.as_str()) != Some("subagent")
                    && a.get("hidden").and_then(|v| v.as_bool()).unwrap_or(false) == false
            })
            .and_then(|a| a.get("name").and_then(|v| v.as_str()).map(|s| s.to_string()))
    }

    fn fallback_model(&self) -> Option<ModelSelection> {
        if let Some(ref model) = self.args.model {
            let parsed = parse_model(model);
            if self.is_model_valid(&parsed) {
                return Some(parsed);
            }
        }
        let store = self.sync.data.lock().unwrap();
        if let Some(config_model) = store.config.get("model").and_then(|v| v.as_str()) {
            let parsed = parse_model(config_model);
            drop(store);
            if self.is_model_valid(&parsed) {
                return Some(parsed);
            }
        }
        let model_store = self.model.lock().unwrap();
        for item in &model_store.recent {
            if self.is_model_valid(item) {
                return Some(item.clone());
            }
        }
        drop(model_store);
        let store = self.sync.data.lock().unwrap();
        let provider = store.provider.first()?;
        let provider_id = provider.get("id").and_then(|v| v.as_str())?.to_string();
        let default_model = store.provider_default.get(&provider_id).cloned();
        let first_model = provider
            .get("models")
            .and_then(|m| m.as_object())
            .and_then(|m| m.keys().next().cloned());
        let model_id = default_model.or(first_model)?;
        Some(ModelSelection {
            provider_id,
            model_id,
        })
    }

    pub fn set_model(&self, model: ModelSelection, recent: bool) {
        if !self.is_model_valid(&model) {
            return;
        }
        let agent = match self.current_agent_name() {
            Some(a) => a,
            None => return,
        };
        let mut store = self.model.lock().unwrap();
        store.model.insert(agent, model.clone());
        if recent {
            store.recent = recent_models(&model, &store.recent);
        }
        let snapshot = store.clone();
        drop(store);
        save_json(&self.model_file, &snapshot);
    }

    pub fn toggle_favorite(&self, model: ModelSelection) {
        if !self.is_model_valid(&model) {
            return;
        }
        let mut store = self.model.lock().unwrap();
        let exists = store.favorite.iter().any(|f| {
            f.provider_id == model.provider_id && f.model_id == model.model_id
        });
        if exists {
            store.favorite.retain(|f| {
                f.provider_id != model.provider_id || f.model_id != model.model_id
            });
        } else {
            store.favorite.insert(0, model);
        }
        let snapshot = store.clone();
        drop(store);
        save_json(&self.model_file, &snapshot);
    }

    pub fn cycle_model(&self, direction: i32) {
        let store = self.model.lock().unwrap();
        let current = self.current_model();
        if current.is_none() {
            return;
        }
        let current = current.unwrap();
        let idx = store.recent.iter().position(|m| {
            m.provider_id == current.provider_id && m.model_id == current.model_id
        });
        if idx.is_none() {
            return;
        }
        let idx = idx.unwrap() as i32;
        let len = store.recent.len() as i32;
        let next = if direction > 0 {
            (idx + 1) % len
        } else {
            (idx - 1 + len) % len
        };
        let next_model = store.recent[next as usize].clone();
        let agent = self.current_agent_name();
        drop(store);
        if let Some(agent) = agent {
            self.model.lock().unwrap().model.insert(agent, next_model);
        }
    }

    pub fn is_pinned(&self, session_id: &str) -> bool {
        self.pinned.lock().unwrap().contains(&session_id.to_string())
    }

    pub fn toggle_pin(&self, session_id: &str) {
        let mut pinned = self.pinned.lock().unwrap();
        let exists = pinned.contains(&session_id.to_string());
        if exists {
            pinned.retain(|x| x != session_id);
        } else {
            pinned.push(session_id.to_string());
        }
        let snapshot = PinnedFile {
            pinned: pinned.clone(),
        };
        drop(pinned);
        save_json(&self.session_file, &snapshot);
    }

    pub fn pinned_slots(&self) -> Vec<String> {
        let pinned = self.pinned.lock().unwrap().clone();
        let store = self.sync.data.lock().unwrap();
        let existing: HashSet<String> = store
            .session
            .iter()
            .filter(|s| s.get("parentID").is_none())
            .filter_map(|s| s.get("id").and_then(|v| v.as_str()).map(|s| s.to_string()))
            .collect();
        pinned.into_iter().filter(|id| existing.contains(id)).take(9).collect()
    }

    pub fn mcp_is_enabled(&self, name: &str) -> bool {
        let store = self.sync.data.lock().unwrap();
        store
            .mcp
            .get(name)
            .and_then(|s| s.get("status").and_then(|v| v.as_str()))
            == Some("connected")
    }
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
struct SessionFile {
    #[serde(default)]
    pinned: Vec<String>,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct PinnedFile {
    pinned: Vec<String>,
}

fn load_json<T: serde::de::DeserializeOwned>(path: &PathBuf) -> Option<T> {
    let content = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

fn save_json<T: serde::Serialize>(path: &PathBuf, value: &T) {
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(json) = serde_json::to_string_pretty(value) {
        let _ = std::fs::write(path, json);
    }
}
