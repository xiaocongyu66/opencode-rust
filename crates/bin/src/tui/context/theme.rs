use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeJson {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub dark: Option<ThemeMode>,
    #[serde(default)]
    pub light: Option<ThemeMode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeMode {
    #[serde(default)]
    pub background: Option<String>,
    #[serde(default)]
    pub foreground: Option<String>,
    #[serde(default)]
    pub primary: Option<String>,
    #[serde(default)]
    pub secondary: Option<String>,
    #[serde(default)]
    pub accent: Option<String>,
    #[serde(default)]
    pub success: Option<String>,
    #[serde(default)]
    pub warning: Option<String>,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub info: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ThemeModeKind {
    Dark,
    Light,
}

#[derive(Debug, Clone)]
pub struct ThemeValues {
    pub background: String,
    pub foreground: String,
    pub primary: String,
    pub secondary: String,
    pub accent: String,
    pub success: String,
    pub warning: String,
    pub error: String,
    pub info: String,
}

pub struct ThemeStore {
    pub themes: HashMap<String, ThemeJson>,
    pub mode: ThemeModeKind,
    pub lock: Option<ThemeModeKind>,
    pub active: String,
    pub ready: bool,
}

impl Default for ThemeStore {
    fn default() -> Self {
        let mut themes = HashMap::new();
        themes.insert(
            "opencode".to_string(),
            ThemeJson {
                name: Some("opencode".to_string()),
                dark: Some(ThemeMode {
                    background: Some("#0a0a0a".to_string()),
                    foreground: Some("#e0e0e0".to_string()),
                    primary: Some("#7c3aed".to_string()),
                    secondary: Some("#3b82f6".to_string()),
                    accent: Some("#ec4899".to_string()),
                    success: Some("#22c55e".to_string()),
                    warning: Some("#f59e0b".to_string()),
                    error: Some("#ef4444".to_string()),
                    info: Some("#06b6d4".to_string()),
                }),
                light: None,
            },
        );
        Self {
            themes,
            mode: ThemeModeKind::Dark,
            lock: None,
            active: "opencode".to_string(),
            ready: false,
        }
    }
}

pub struct ThemeContext {
    pub store: Arc<Mutex<ThemeStore>>,
}

impl ThemeContext {
    pub fn new(mode: ThemeModeKind) -> Self {
        let store = ThemeStore {
            mode,
            ..Default::default()
        };
        Self {
            store: Arc::new(Mutex::new(store)),
        }
    }

    pub fn theme(&self) -> ThemeValues {
        let store = self.store.lock().unwrap();
        let theme = store
            .themes
            .get(&store.active)
            .or_else(|| store.themes.get("opencode"));
        let mode_data = match &store.mode {
            ThemeModeKind::Dark => theme.as_ref().and_then(|t| t.dark.as_ref()),
            ThemeModeKind::Light => theme.as_ref().and_then(|t| t.light.as_ref()),
        };
        let mode_data = mode_data.cloned().unwrap_or_default();
        ThemeValues {
            background: mode_data.background.unwrap_or_else(|| "#0a0a0a".to_string()),
            foreground: mode_data.foreground.unwrap_or_else(|| "#e0e0e0".to_string()),
            primary: mode_data.primary.unwrap_or_else(|| "#7c3aed".to_string()),
            secondary: mode_data.secondary.unwrap_or_else(|| "#3b82f6".to_string()),
            accent: mode_data.accent.unwrap_or_else(|| "#ec4899".to_string()),
            success: mode_data.success.unwrap_or_else(|| "#22c55e".to_string()),
            warning: mode_data.warning.unwrap_or_else(|| "#f59e0b".to_string()),
            error: mode_data.error.unwrap_or_else(|| "#ef4444".to_string()),
            info: mode_data.info.unwrap_or_else(|| "#06b6d4".to_string()),
        }
    }

    pub fn selected(&self) -> String {
        self.store.lock().unwrap().active.clone()
    }

    pub fn mode(&self) -> ThemeModeKind {
        self.store.lock().unwrap().mode.clone()
    }

    pub fn locked(&self) -> bool {
        self.store.lock().unwrap().lock.is_some()
    }

    pub fn lock(&self) {
        let mut store = self.store.lock().unwrap();
        store.lock = Some(store.mode.clone());
    }

    pub fn unlock(&self) {
        let mut store = self.store.lock().unwrap();
        store.lock = None;
    }

    pub fn set_mode(&self, mode: ThemeModeKind) {
        let mut store = self.store.lock().unwrap();
        store.mode = mode;
        store.lock = Some(mode.clone());
    }

    pub fn set(&self, theme: &str) -> bool {
        let mut store = self.store.lock().unwrap();
        if !store.themes.contains_key(theme) {
            return false;
        }
        store.active = theme.to_string();
        true
    }

    pub fn has(&self, theme: &str) -> bool {
        self.store.lock().unwrap().themes.contains_key(theme)
    }

    pub fn ready(&self) -> bool {
        self.store.lock().unwrap().ready
    }

    pub fn set_ready(&self, ready: bool) {
        self.store.lock().unwrap().ready = ready;
    }

    pub fn set_active(&self, active: String) {
        self.store.lock().unwrap().active = active;
    }

    pub fn add_theme(&self, name: String, theme: ThemeJson) {
        self.store.lock().unwrap().themes.insert(name, theme);
    }

    pub fn all_themes(&self) -> Vec<String> {
        self.store.lock().unwrap().themes.keys().cloned().collect()
    }
}

impl Default for ThemeMode {
    fn default() -> Self {
        Self {
            background: None,
            foreground: None,
            primary: None,
            secondary: None,
            accent: None,
            success: None,
            warning: None,
            error: None,
            info: None,
        }
    }
}

pub fn terminal_mode_from_colors(colors: &serde_json::Value) -> Option<ThemeModeKind> {
    let bg = colors.get("palette").and_then(|p| p.get(0)).and_then(|c| c.as_str());
    match bg {
        Some(bg) => {
            let brightness = parse_color_brightness(bg);
            if brightness < 128 {
                Some(ThemeModeKind::Dark)
            } else {
                Some(ThemeModeKind::Light)
            }
        }
        None => None,
    }
}

fn parse_color_brightness(hex: &str) -> u8 {
    let hex = hex.trim_start_matches('#');
    if hex.len() >= 6 {
        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
        ((r as u32 * 299 + g as u32 * 587 + b as u32 * 114) / 1000) as u8
    } else {
        0
    }
}
