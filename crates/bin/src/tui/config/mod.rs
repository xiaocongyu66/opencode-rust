pub mod keybind;
pub mod keymap;

pub use keybind::{Keybind, KeybindName, BindingValue, LEADER_DEFAULT, COMMAND_MAP, DEFINITIONS, COMMAND_DESCRIPTIONS};
pub use keymap::{Keymap, ModeStack, LEADER_TOKEN, OPENCODE_BASE_MODE, COMMAND_PALETTE_COMMAND};

use std::collections::HashMap;

pub const LEADER_TIMEOUT_DEFAULT: u64 = 2000;

#[derive(Debug, Clone)]
pub struct Attention {
    pub enabled: bool,
    pub notifications: bool,
    pub sound: bool,
    pub volume: f64,
    pub sound_pack: String,
    pub sounds: HashMap<String, Option<String>>,
}

impl Default for Attention {
    fn default() -> Self {
        Self {
            enabled: false,
            notifications: true,
            sound: true,
            volume: 0.4,
            sound_pack: "opencode.default".to_string(),
            sounds: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct PromptConfig {
    pub max_height: Option<i32>,
    pub max_width: Option<PromptMaxWidth>,
}

#[derive(Debug, Clone)]
pub enum PromptMaxWidth {
    Fixed(i32),
    Auto,
}

#[derive(Debug, Clone)]
pub struct CursorConfig {
    pub style: CursorStyle,
    pub blinking: bool,
}

#[derive(Debug, Clone, Default)]
pub enum CursorStyle {
    #[default]
    Block,
    Underline,
    Line,
    Default,
}

#[derive(Debug, Clone, Default)]
pub enum DiffStyle {
    #[default]
    Auto,
    Stacked,
}

#[derive(Debug, Clone)]
pub struct ResolveOptions {
    pub terminal_suspend: bool,
}

#[derive(Debug, Clone)]
pub struct Resolved {
    pub theme: Option<String>,
    pub keybinds: Vec<Keybind>,
    pub leader_timeout: u64,
    pub mouse: bool,
    pub cursor: Option<CursorConfig>,
    pub attention: Attention,
    pub prompt: Option<PromptConfig>,
    pub scroll_speed: Option<f64>,
    pub diff_style: DiffStyle,
}

pub fn resolve(input: &TuiConfigInfo, options: &ResolveOptions) -> Resolved {
    let mut overrides: HashMap<String, BindingValue> = input.keybinds.clone();

    if !options.terminal_suspend {
        overrides.insert(
            "terminal_suspend".to_string(),
            BindingValue::None,
        );

        if !overrides.contains_key("input_undo") {
            let default_undo = keybind::default_value("input_undo");
            if let BindingValue::Items(default_items) = default_undo {
                let mut keys: Vec<String> = vec!["ctrl+z".to_string()];
                for item in &default_items {
                    if let BindingValue::Single(s) = item {
                        for part in s.split(',') {
                            let trimmed = part.trim().to_string();
                            if !keys.contains(&trimmed) {
                                keys.push(trimmed);
                            }
                        }
                    }
                }
                let joined = keys.join(",");
                overrides.insert(
                    "input_undo".to_string(),
                    BindingValue::Items(vec![BindingValue::Single(joined)]),
                );
            }
        }
    }

    let keybinds = keybind::parse(&overrides);

    Resolved {
        theme: input.theme.clone(),
        keybinds,
        leader_timeout: input.leader_timeout.unwrap_or(LEADER_TIMEOUT_DEFAULT),
        mouse: input.mouse.unwrap_or(true),
        cursor: input.cursor.as_ref().map(|c| CursorConfig {
            style: c.style.clone().unwrap_or_default(),
            blinking: c.blinking.unwrap_or(true),
        }),
        attention: Attention {
            enabled: input.attention.as_ref().and_then(|a| a.enabled).unwrap_or(false),
            notifications: input.attention.as_ref().and_then(|a| a.notifications).unwrap_or(true),
            sound: input.attention.as_ref().and_then(|a| a.sound).unwrap_or(true),
            volume: input.attention.as_ref().and_then(|a| a.volume).unwrap_or(0.4),
            sound_pack: input.attention.as_ref().and_then(|a| a.sound_pack.clone()).unwrap_or_else(|| "opencode.default".to_string()),
            sounds: input.attention.as_ref().map(|a| a.sounds.clone()).unwrap_or_default(),
        },
        prompt: input.prompt.clone(),
        scroll_speed: input.scroll_speed,
        diff_style: input.diff_style.clone().unwrap_or_default(),
    }
}

#[derive(Debug, Clone, Default)]
pub struct TuiConfigInfo {
    pub theme: Option<String>,
    pub keybinds: HashMap<String, BindingValue>,
    pub leader_timeout: Option<u64>,
    pub mouse: Option<bool>,
    pub cursor: Option<TuiCursorInput>,
    pub attention: Option<AttentionInput>,
    pub prompt: Option<PromptConfig>,
    pub scroll_speed: Option<f64>,
    pub diff_style: Option<DiffStyle>,
}

#[derive(Debug, Clone, Default)]
pub struct TuiCursorInput {
    pub style: Option<CursorStyle>,
    pub blinking: Option<bool>,
}

#[derive(Debug, Clone, Default)]
pub struct AttentionInput {
    pub enabled: Option<bool>,
    pub notifications: Option<bool>,
    pub sound: Option<bool>,
    pub volume: Option<f64>,
    pub sound_pack: Option<String>,
    pub sounds: HashMap<String, Option<String>>,
}
