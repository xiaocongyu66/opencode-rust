use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

use super::keybind::{
    all_default_keybinds, BindingValue, Keybind, KeybindCategory,
};

pub const LEADER_TOKEN: &str = "leader";
pub const OPENCODE_BASE_MODE: &str = "base";
pub const COMMAND_PALETTE_COMMAND: &str = "command.palette.show";

const OPENCODE_MODE_KEY: &str = "opencode.mode";

static MODE_ID_COUNTER: AtomicU64 = AtomicU64::new(0);

pub static KEY_ALIASES: &[(&str, &str)] = &[
    ("enter", "return"),
    ("esc", "escape"),
    ("pgdown", "pagedown"),
    ("pgup", "pageup"),
];

pub fn expand_key_aliases(input: &str) -> Option<String> {
    let mut result = input.to_string();
    for (alias, key) in KEY_ALIASES {
        let pattern_lower = format!("{}", alias);
        let replacement = format!("{}", key);
        let lower = result.to_lowercase();
        let mut new_result = String::new();
        let mut last_end = 0;
        let mut changed = false;
        let bytes = lower.as_bytes();
        let alias_bytes = pattern_lower.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            if i + alias_bytes.len() <= bytes.len() && &bytes[i..i + alias_bytes.len()] == alias_bytes {
                let is_left_boundary = i == 0
                    || matches!(bytes[i - 1], b'+' | b',' | b' ' | b'>');
                let right_idx = i + alias_bytes.len();
                let is_right_boundary = right_idx >= bytes.len()
                    || matches!(bytes[right_idx], b'+' | b',' | b' ' | b'<');
                if is_left_boundary && is_right_boundary {
                    new_result.push_str(&result[last_end..i]);
                    new_result.push_str(&replacement);
                    i = right_idx;
                    last_end = i;
                    changed = true;
                    continue;
                }
            }
            i += 1;
        }
        new_result.push_str(&result[last_end..]);
        if changed {
            result = new_result;
        }
    }
    if result == input { None } else { Some(result) }
}

#[derive(Debug, Clone)]
pub struct RegisteredCommand {
    pub name: String,
    pub key: String,
    pub description: String,
    pub category: KeybindCategory,
    pub hidden: bool,
    pub slash_name: Option<String>,
    pub slash_aliases: Vec<String>,
    pub title: Option<String>,
}

#[derive(Debug, Clone)]
struct ModeFrame {
    id: u64,
    mode: String,
    active: bool,
}

#[derive(Debug)]
pub struct ModeStack {
    frames: Vec<ModeFrame>,
    current_mode: String,
    disposed: bool,
}

impl ModeStack {
    pub fn new() -> Self {
        Self {
            frames: Vec::new(),
            current_mode: OPENCODE_BASE_MODE.to_string(),
            disposed: false,
        }
    }

    pub fn current(&self) -> &str {
        if self.frames.is_empty() {
            &self.current_mode
        } else {
            &self.frames.last().unwrap().mode
        }
    }

    pub fn push(&mut self, mode: &str) -> u64 {
        if self.disposed {
            return 0;
        }
        let id = MODE_ID_COUNTER.fetch_add(1, Ordering::SeqCst);
        self.frames.push(ModeFrame {
            id,
            mode: mode.to_string(),
            active: true,
        });
        self.current_mode = mode.to_string();
        id
    }

    pub fn pop(&mut self, id: u64) {
        if self.disposed {
            return;
        }
        if let Some(idx) = self.frames.iter().position(|f| f.id == id && f.active) {
            self.frames.remove(idx);
            self.frames.last_mut().map(|f| f.active = true);
            self.current_mode = self
                .frames
                .last()
                .map(|f| f.mode.clone())
                .unwrap_or_else(|| OPENCODE_BASE_MODE.to_string());
        }
    }

    pub fn dispose(&mut self) {
        if self.disposed {
            return;
        }
        self.disposed = true;
        self.frames.clear();
        self.current_mode = String::new();
    }

    pub fn is_disposed(&self) -> bool {
        self.disposed
    }
}

impl Default for ModeStack {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct Keymap {
    commands: HashMap<String, RegisteredCommand>,
    keybinds: Vec<Keybind>,
    mode_stack: ModeStack,
    data: HashMap<String, String>,
    leader_timeout: u64,
}

impl Keymap {
    pub fn new() -> Self {
        let keybinds = all_default_keybinds();
        let mut keymap = Self {
            commands: HashMap::new(),
            keybinds,
            mode_stack: ModeStack::new(),
            data: HashMap::new(),
            leader_timeout: 2000,
        };
        keymap.data.insert(
            OPENCODE_MODE_KEY.to_string(),
            OPENCODE_BASE_MODE.to_string(),
        );
        keymap.register_default_commands();
        keymap
    }

    pub fn with_leader_timeout(mut self, timeout: u64) -> Self {
        self.leader_timeout = timeout;
        self
    }

    pub fn with_keybinds(mut self, keybinds: Vec<Keybind>) -> Self {
        self.keybinds = keybinds;
        self.register_default_commands();
        self
    }

    fn register_default_commands(&mut self) {
        for kb in &self.keybinds {
            self.commands.insert(
                kb.command.to_string(),
                RegisteredCommand {
                    name: kb.command.to_string(),
                    key: format_binding_value(&kb.default),
                    description: kb.description.to_string(),
                    category: kb.category,
                    hidden: false,
                    slash_name: None,
                    slash_aliases: Vec::new(),
                    title: None,
                },
            );
        }
    }

    pub fn register_command(
        &mut self,
        name: &str,
        key: &str,
        description: &str,
        category: KeybindCategory,
    ) {
        self.commands.insert(
            name.to_string(),
            RegisteredCommand {
                name: name.to_string(),
                key: key.to_string(),
                description: description.to_string(),
                category,
                hidden: false,
                slash_name: None,
                slash_aliases: Vec::new(),
                title: None,
            },
        );
    }

    pub fn register_command_with_options(
        &mut self,
        name: &str,
        key: &str,
        description: &str,
        category: KeybindCategory,
        hidden: bool,
        slash_name: Option<String>,
        slash_aliases: Vec<String>,
        title: Option<String>,
    ) {
        self.commands.insert(
            name.to_string(),
            RegisteredCommand {
                name: name.to_string(),
                key: key.to_string(),
                description: description.to_string(),
                category,
                hidden,
                slash_name,
                slash_aliases,
                title,
            },
        );
    }

    pub fn get_command(&self, name: &str) -> Option<&RegisteredCommand> {
        self.commands.get(name)
    }

    pub fn get_commands(&self) -> Vec<&RegisteredCommand> {
        self.commands.values().collect()
    }

    pub fn get_visible_commands(&self) -> Vec<&RegisteredCommand> {
        self.commands
            .values()
            .filter(|cmd| !cmd.hidden && cmd.name != COMMAND_PALETTE_COMMAND)
            .collect()
    }

    pub fn get_command_entries(&self, namespace: &str) -> Vec<&RegisteredCommand> {
        self.commands
            .values()
            .filter(|cmd| match namespace {
                "palette" => !cmd.hidden && cmd.name != COMMAND_PALETTE_COMMAND,
                _ => true,
            })
            .collect()
    }

    pub fn find_keybind(&self, command: &str) -> Option<&Keybind> {
        self.keybinds.iter().find(|kb| kb.command == command)
    }

    pub fn find_keybind_by_name(&self, name: &str) -> Option<&Keybind> {
        self.keybinds.iter().find(|kb| kb.name == name)
    }

    pub fn get_keys_for_command(&self, command: &str) -> Vec<String> {
        match self.find_keybind(command) {
            Some(kb) => kb.default.as_keys(),
            None => match self.commands.get(command) {
                Some(cmd) if !cmd.key.is_empty() && cmd.key != "none" => {
                    cmd.key.split(',').map(|s| s.trim().to_string()).collect()
                }
                _ => Vec::new(),
            },
        }
    }

    pub fn resolve_command(&self, key_sequence: &str) -> Option<&RegisteredCommand> {
        let expanded = expand_key_aliases(key_sequence);
        let search = expanded.as_deref().unwrap_or(key_sequence);
        self.commands
            .values()
            .find(|cmd| cmd.key == search || cmd.key == key_sequence)
    }

    pub fn dispatch_command(&self, name: &str) -> bool {
        self.commands.contains_key(name)
    }

    pub fn mode_stack(&mut self) -> &mut ModeStack {
        &mut self.mode_stack
    }

    pub fn current_mode(&self) -> &str {
        self.mode_stack.current()
    }

    pub fn push_mode(&mut self, mode: &str) -> u64 {
        let id = self.mode_stack.push(mode);
        self.data.insert(OPENCODE_MODE_KEY.to_string(), mode.to_string());
        id
    }

    pub fn pop_mode(&mut self, id: u64) {
        self.mode_stack.pop(id);
        self.data.insert(
            OPENCODE_MODE_KEY.to_string(),
            self.mode_stack.current().to_string(),
        );
    }

    pub fn set_data(&mut self, key: &str, value: Option<String>) {
        match value {
            Some(v) => {
                self.data.insert(key.to_string(), v);
            }
            None => {
                self.data.remove(key);
            }
        }
    }

    pub fn get_data(&self, key: &str) -> Option<&String> {
        self.data.get(key)
    }

    pub fn leader_timeout(&self) -> u64 {
        self.leader_timeout
    }

    pub fn leader_key(&self) -> Option<String> {
        self.find_keybind_by_name(LEADER_TOKEN)
            .map(|kb| kb.default.as_keys().first().cloned())
            .flatten()
            .or_else(|| Some(super::keybind::LEADER_DEFAULT.to_string()))
    }

    pub fn leader_display(&self) -> String {
        match self.find_keybind_by_name(LEADER_TOKEN) {
            Some(kb) => {
                let keys = kb.default.as_keys();
                if keys.is_empty() {
                    super::keybind::LEADER_DEFAULT.to_string()
                } else {
                    keys[0].clone()
                }
            }
            None => super::keybind::LEADER_DEFAULT.to_string(),
        }
    }

    pub fn all_keybinds(&self) -> &[Keybind] {
        &self.keybinds
    }

    pub fn keybinds_by_category(&self, category: KeybindCategory) -> Vec<&Keybind> {
        self.keybinds
            .iter()
            .filter(|kb| kb.category == category)
            .collect()
    }

    pub fn format_key_sequence(parts: &[String]) -> String {
        parts.join(" ")
    }

    pub fn format_command_bindings(command: &str, keymap: &Keymap) -> String {
        keymap
            .get_keys_for_command(command)
            .join(", ")
    }

    pub fn gather(&self, _prefix: &str, commands: &[&str]) -> Vec<(&Keybind, &RegisteredCommand)> {
        commands
            .iter()
            .filter_map(|cmd| {
                let kb = self.find_keybind(cmd)?;
                let rc = self.get_command(cmd)?;
                Some((kb, rc))
            })
            .collect()
    }

    pub fn pick(&self, _name: &str, commands: &[&str]) -> Vec<&Keybind> {
        commands
            .iter()
            .filter_map(|cmd| self.find_keybind(cmd))
            .collect()
    }

    pub fn omit(&self, _name: &str, commands: &[&str]) -> Vec<&Keybind> {
        let exclude: Vec<&str> = commands.to_vec();
        self.keybinds
            .iter()
            .filter(|kb| !exclude.contains(&kb.command))
            .collect()
    }

    pub fn get_pending_sequence(&self) -> Vec<String> {
        Vec::new()
    }

    pub fn is_leader_active(&self) -> bool {
        false
    }
}

impl Default for Keymap {
    fn default() -> Self {
        Self::new()
    }
}

fn format_binding_value(value: &BindingValue) -> String {
    match value {
        BindingValue::Disabled | BindingValue::None => "none".to_string(),
        BindingValue::Single(s) => s.clone(),
        BindingValue::Items(items) => items
            .iter()
            .map(|v| format_binding_value(v))
            .collect::<Vec<_>>()
            .join(","),
    }
}

pub static INPUT_COMMANDS: &[&str] = &[
    "input.move.left",
    "input.move.right",
    "input.move.up",
    "input.move.down",
    "input.select.left",
    "input.select.right",
    "input.select.up",
    "input.select.down",
    "input.line.home",
    "input.line.end",
    "input.select.line.home",
    "input.select.line.end",
    "input.visual.line.home",
    "input.visual.line.end",
    "input.select.visual.line.home",
    "input.select.visual.line.end",
    "input.buffer.home",
    "input.buffer.end",
    "input.select.buffer.home",
    "input.select.buffer.end",
    "input.delete.line",
    "input.delete.to.line.end",
    "input.delete.to.line.start",
    "input.backspace",
    "input.delete",
    "input.newline",
    "input.undo",
    "input.redo",
    "input.word.forward",
    "input.word.backward",
    "input.select.word.forward",
    "input.select.word.backward",
    "input.delete.word.forward",
    "input.delete.word.backward",
    "input.select.all",
    "input.submit",
];

#[derive(Debug, Clone)]
pub struct CommandSlashEntry {
    pub display: String,
    pub description: Option<String>,
    pub aliases: Vec<String>,
    pub command_name: String,
}

pub fn command_slashes(keymap: &Keymap) -> Vec<CommandSlashEntry> {
    keymap
        .get_command_entries("palette")
        .into_iter()
        .filter_map(|cmd| {
            let slash_name = cmd.slash_name.as_ref()?;
            if slash_name.is_empty() {
                return None;
            }
            Some(CommandSlashEntry {
                display: format!("/{}", slash_name),
                description: if !cmd.description.is_empty() {
                    Some(cmd.description.clone())
                } else {
                    cmd.title.clone()
                },
                aliases: cmd
                    .slash_aliases
                    .iter()
                    .filter(|a| !a.is_empty())
                    .map(|a| format!("/{}", a))
                    .collect(),
                command_name: cmd.name.clone(),
            })
        })
        .collect()
}

pub fn is_visible_palette_command(cmd: &RegisteredCommand) -> bool {
    !cmd.hidden && cmd.name != COMMAND_PALETTE_COMMAND
}
