use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

pub struct KvContext {
    store: Arc<Mutex<HashMap<String, serde_json::Value>>>,
    file_path: PathBuf,
    ready: Arc<Mutex<bool>>,
}

impl KvContext {
    pub fn new(state_dir: PathBuf) -> Self {
        let file_path = state_dir.join("kv.json");
        let store = match std::fs::read_to_string(&file_path) {
            Ok(content) => serde_json::from_str::<HashMap<String, serde_json::Value>>(&content)
                .unwrap_or_default(),
            Err(_) => HashMap::new(),
        };
        let ready = !store.is_empty() || file_path.exists();
        Self {
            store: Arc::new(Mutex::new(store)),
            file_path,
            ready: Arc::new(Mutex::new(ready)),
        }
    }

    pub fn ready(&self) -> bool {
        *self.ready.lock().unwrap()
    }

    pub fn get(&self, key: &str) -> Option<serde_json::Value> {
        self.store.lock().unwrap().get(key).cloned()
    }

    pub fn get_or(&self, key: &str, default: serde_json::Value) -> serde_json::Value {
        self.store
            .lock()
            .unwrap()
            .get(key)
            .cloned()
            .unwrap_or(default)
    }

    pub fn get_bool(&self, key: &str, default: bool) -> bool {
        self.get(key)
            .and_then(|v| v.as_bool())
            .unwrap_or(default)
    }

    pub fn set(&self, key: &str, value: serde_json::Value) {
        {
            let mut store = self.store.lock().unwrap();
            store.insert(key.to_string(), value);
        }
        self.persist();
    }

    pub fn signal(&self, key: &str, default: serde_json::Value) -> serde_json::Value {
        let mut store = self.store.lock().unwrap();
        store.entry(key.to_string()).or_insert(default).clone()
    }

    fn persist(&self) {
        let snapshot = self.store.lock().unwrap().clone();
        let path = self.file_path.clone();
        std::thread::spawn(move || {
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            if let Ok(json) = serde_json::to_string_pretty(&snapshot) {
                let _ = std::fs::write(&path, json);
            }
        });
    }
}

#[derive(Serialize, Deserialize)]
struct KvFile {
    #[serde(flatten)]
    data: HashMap<String, serde_json::Value>,
}
