//! TUI attention — notification, sound, and focus management.
//! Ported from tui/src/attention.ts (260 lines)
//!
//! Features:
//! - Focus state tracking (unknown/focused/blurred)
//! - Terminal bell / OSC 9 notification
//! - Sound pack registry (builtin + custom)
//! - Configurable notification timing (always/blurred/focused)

use std::collections::HashMap;
use std::io::{self, Write};

const DEFAULT_TITLE: &str = "opencode";
const DEFAULT_PACK_ID: &str = "opencode.default";
const KV_SOUND_PACK: &str = "attention_sound_pack";
const TITLE_LIMIT: usize = 80;
const MESSAGE_LIMIT: usize = 240;

/// Sound names recognized by the attention system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SoundName {
    Default,
    Question,
    Permission,
    Error,
    Done,
    SubagentDone,
}

impl SoundName {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "default" => Some(Self::Default),
            "question" => Some(Self::Question),
            "permission" => Some(Self::Permission),
            "error" => Some(Self::Error),
            "done" => Some(Self::Done),
            "subagent_done" => Some(Self::SubagentDone),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Question => "question",
            Self::Permission => "permission",
            Self::Error => "error",
            Self::Done => "done",
            Self::SubagentDone => "subagent_done",
        }
    }
}

/// When to trigger a notification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttentionWhen {
    Always,
    Blurred,
    Focused,
}

impl AttentionWhen {
    pub fn from_str(s: &str) -> Self {
        match s {
            "always" => Self::Always,
            "blurred" => Self::Blurred,
            "focused" => Self::Focused,
            _ => Self::Blurred,
        }
    }
}

/// Focus state of the terminal window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusState {
    Unknown,
    Focused,
    Blurred,
}

/// Skip reason for a notification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotifySkipReason {
    AttentionDisabled,
    RendererDestroyed,
    EmptyMessage,
    FocusUnknown,
    Focused,
    Blurred,
}

/// Result of a notify call.
#[derive(Debug, Clone)]
pub struct NotifyResult {
    pub ok: bool,
    pub notification: bool,
    pub sound: bool,
    pub skipped: Option<NotifySkipReason>,
}

impl NotifyResult {
    fn skipped(reason: NotifySkipReason) -> Self {
        Self {
            ok: false,
            notification: false,
            sound: false,
            skipped: Some(reason),
        }
    }

    fn failed() -> Self {
        Self {
            ok: false,
            notification: false,
            sound: false,
            skipped: None,
        }
    }
}

/// Input for a notify request.
#[derive(Debug, Clone)]
pub struct NotifyInput {
    pub message: String,
    pub title: Option<String>,
    pub notification: NotifyConfig,
    pub sound: SoundConfig,
}

/// Notification configuration.
#[derive(Debug, Clone)]
pub enum NotifyConfig {
    Default,
    Disabled,
    When(AttentionWhen),
}

/// Sound configuration.
#[derive(Debug, Clone)]
pub enum SoundConfig {
    Disabled,
    Default,
    Named(SoundName, Option<f64>, Option<AttentionWhen>),
}

/// Attention configuration — mirrors `TuiConfig.Resolved["attention"]`.
#[derive(Debug, Clone)]
pub struct AttentionConfig {
    pub enabled: bool,
    pub notifications: bool,
    pub sound: bool,
    pub volume: f64,
    pub sound_pack: Option<String>,
    pub sounds: HashMap<SoundName, String>,
}

impl Default for AttentionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            notifications: true,
            sound: false,
            volume: 0.5,
            sound_pack: None,
            sounds: HashMap::new(),
        }
    }
}

/// A registered sound pack.
#[derive(Debug, Clone)]
pub struct SoundPack {
    pub id: String,
    pub name: Option<String>,
    pub builtin: bool,
    pub sounds: HashMap<SoundName, String>,
}

/// Sound pack info for listing.
#[derive(Debug, Clone)]
pub struct SoundPackInfo {
    pub id: String,
    pub name: Option<String>,
    pub active: bool,
    pub builtin: bool,
}

/// Builtin sound pack — paths reference the bundled audio assets.
fn builtin_pack() -> SoundPack {
    let mut sounds = HashMap::new();
    sounds.insert(SoundName::Default, "audio/bip-bop-01.mp3".to_string());
    sounds.insert(SoundName::Question, "audio/bip-bop-03.mp3".to_string());
    sounds.insert(SoundName::Permission, "audio/staplebops-06.mp3".to_string());
    sounds.insert(SoundName::Error, "audio/nope-03.mp3".to_string());
    sounds.insert(SoundName::Done, "audio/bip-bop-01.mp3".to_string());
    sounds.insert(SoundName::SubagentDone, "audio/yup-01.mp3".to_string());
    SoundPack {
        id: DEFAULT_PACK_ID.to_string(),
        name: Some("OpenCode Default".to_string()),
        builtin: true,
        sounds,
    }
}

/// Simple KV store trait — mirrors `TuiKV`.
pub trait KvStore {
    fn get(&self, key: &str, default: &str) -> String;
    fn set(&mut self, key: &str, value: &str);
}

/// A no-op KV store for when persistence is not needed.
pub struct NoopKv;

impl KvStore for NoopKv {
    fn get(&self, _key: &str, default: &str) -> String {
        default.to_string()
    }
    fn set(&mut self, _key: &str, _value: &str) {}
}

/// TUI attention host — the main attention manager.
pub struct TuiAttention {
    config: AttentionConfig,
    focus: FocusState,
    disposed: bool,
    destroyed: bool,
    packs: HashMap<String, SoundPack>,
    active_pack_id: Option<String>,
    kv: Option<Box<dyn KvStore>>,
}

impl TuiAttention {
    pub fn new(config: AttentionConfig, kv: Option<Box<dyn KvStore>>) -> Self {
        let builtin = builtin_pack();
        let mut packs = HashMap::new();
        packs.insert(builtin.id.clone(), builtin);
        Self {
            config,
            focus: FocusState::Unknown,
            disposed: false,
            destroyed: false,
            packs,
            active_pack_id: None,
            kv,
        }
    }

    pub fn set_focus(&mut self, focused: bool) {
        self.focus = if focused {
            FocusState::Focused
        } else {
            FocusState::Blurred
        };
    }

    pub fn mark_destroyed(&mut self) {
        self.destroyed = true;
    }

    fn configured_pack_id(&self) -> String {
        if let Some(active) = &self.active_pack_id {
            return active.clone();
        }
        if let Some(kv) = &self.kv {
            let stored = kv.get(KV_SOUND_PACK, "");
            if !stored.is_empty() {
                return stored;
            }
        }
        self.config
            .sound_pack
            .clone()
            .unwrap_or_else(|| DEFAULT_PACK_ID.to_string())
    }

    fn current_pack(&self) -> &SoundPack {
        let id = self.configured_pack_id();
        self.packs.get(&id).unwrap_or_else(|| {
            self.packs
                .get(DEFAULT_PACK_ID)
                .expect("builtin pack always exists")
        })
    }

    fn sound_candidates(&self, name: SoundName) -> Vec<String> {
        let mut candidates = Vec::new();
        if let Some(path) = self.config.sounds.get(&name) {
            if !path.trim().is_empty() && !candidates.contains(path) {
                candidates.push(path.clone());
            }
        }
        if let Some(path) = self.current_pack().sounds.get(&name) {
            if !path.trim().is_empty() && !candidates.contains(path) {
                candidates.push(path.clone());
            }
        }
        if let Some(builtin) = self.packs.get(DEFAULT_PACK_ID) {
            if let Some(path) = builtin.sounds.get(&name) {
                if !path.trim().is_empty() && !candidates.contains(path) {
                    candidates.push(path.clone());
                }
            }
        }
        candidates
    }

    fn play_sound(&self, name: SoundName, _volume: f64) -> bool {
        let candidates = self.sound_candidates(name);
        if candidates.is_empty() {
            return false;
        }
        // Audio playback would require an audio backend.
        // On headless terminals, this is a no-op.
        tracing::debug!("attempting to play sound: {:?}", candidates.first());
        false
    }

    fn focus_skip(&self, when: AttentionWhen, focus: FocusState) -> Option<NotifySkipReason> {
        match (when, focus) {
            (AttentionWhen::Always, _) => None,
            (_, FocusState::Unknown) => Some(NotifySkipReason::FocusUnknown),
            (AttentionWhen::Blurred, FocusState::Focused) => Some(NotifySkipReason::Focused),
            (AttentionWhen::Focused, FocusState::Blurred) => Some(NotifySkipReason::Blurred),
            _ => None,
        }
    }

    fn sound_volume(&self, input: &NotifyInput) -> Option<f64> {
        if !self.config.sound {
            return None;
        }
        match &input.sound {
            SoundConfig::Disabled => None,
            SoundConfig::Default => Some(clamp_volume(self.config.volume)),
            SoundConfig::Named(_, volume_override, _) => {
                Some(clamp_volume(volume_override.unwrap_or(self.config.volume)))
            }
        }
    }

    fn trigger_notification(&self, message: &str, title: &str) -> bool {
        // OSC 9: terminal notification (iTerm2, kitty, etc.)
        let mut stdout = io::stdout();
        let _ = write!(stdout, "\x1b]9;{}|{}\x07", title, message);
        let _ = stdout.flush();
        // Also try OSC 777 for tmux-compatible notifications
        let _ = write!(stdout, "\x1b]777;notify;{};{}\x07", title, message);
        let _ = stdout.flush();
        true
    }

    /// Send a notification — main entry point.
    pub fn notify(&mut self, request: &NotifyInput) -> NotifyResult {
        if !self.config.enabled {
            return NotifyResult::skipped(NotifySkipReason::AttentionDisabled);
        }
        if self.disposed || self.destroyed {
            return NotifyResult::skipped(NotifySkipReason::RendererDestroyed);
        }

        let message = normalize_text(&request.message, "", MESSAGE_LIMIT);
        if message.is_empty() {
            return NotifyResult::skipped(NotifySkipReason::EmptyMessage);
        }

        let notification_when = match &request.notification {
            NotifyConfig::Disabled => return {
                let volume = self.sound_volume(request);
                let sound = match volume {
                    Some(v) => {
                        let sound_name = match &request.sound {
                            SoundConfig::Named(name, _, _) => *name,
                            _ => SoundName::Default,
                        };
                        let sound_when = match &request.sound {
                            SoundConfig::Named(_, _, Some(w)) => *w,
                            _ => AttentionWhen::Always,
                        };
                        if self.focus_skip(sound_when, self.focus).is_some() {
                            false
                        } else {
                            self.play_sound(sound_name, v)
                        }
                    }
                    None => false,
                };
                return NotifyResult {
                    ok: sound,
                    notification: false,
                    sound,
                    skipped: if !sound { Some(NotifySkipReason::Focused) } else { None },
                };
            },
            NotifyConfig::Default => AttentionWhen::Blurred,
            NotifyConfig::When(w) => *w,
        };

        let notification_requested = self.config.notifications;
        let notification_skip = self.focus_skip(notification_when, self.focus);
        let should_notify = notification_requested && notification_skip.is_none();
        let notification = if should_notify {
            let title = normalize_text(
                request.title.as_deref().unwrap_or(DEFAULT_TITLE),
                DEFAULT_TITLE,
                TITLE_LIMIT,
            );
            self.trigger_notification(&message, &title)
        } else {
            false
        };

        let volume = self.sound_volume(request);
        let (sound_name, sound_when) = match &request.sound {
            SoundConfig::Disabled => (SoundName::Default, AttentionWhen::Always),
            SoundConfig::Default => (SoundName::Default, AttentionWhen::Always),
            SoundConfig::Named(name, _, when_override) => {
                (*name, when_override.unwrap_or(AttentionWhen::Always))
            }
        };
        let sound_skip = if volume.is_some() {
            self.focus_skip(sound_when, self.focus)
        } else {
            None
        };
        let sound = if let Some(v) = volume {
            if sound_skip.is_some() {
                false
            } else {
                self.play_sound(sound_name, v)
            }
        } else {
            false
        };

        if !notification && !sound {
            if let Some(reason) = notification_skip {
                return NotifyResult::skipped(reason);
            }
            if let Some(reason) = sound_skip {
                return NotifyResult::skipped(reason);
            }
        }

        NotifyResult {
            ok: notification || sound,
            notification,
            sound,
            skipped: None,
        }
    }

    /// Register a custom sound pack.
    pub fn register_pack(&mut self, pack: SoundPack) -> impl FnOnce() + 'static {
        self.packs.insert(pack.id.clone(), pack);
        let packs = &mut self.packs as *mut HashMap<String, SoundPack>;
        let pack_id = self.packs.keys().last().cloned().unwrap_or_default();
        let mut disposed = false;
        move || {
            if disposed {
                return;
            }
            disposed = true;
            unsafe {
                (*packs).remove(&pack_id);
            }
        }
    }

    /// Activate a sound pack by ID.
    pub fn activate_pack(&mut self, id: &str, persist: bool) -> bool {
        if !self.packs.contains_key(id) {
            return false;
        }
        self.active_pack_id = Some(id.to_string());
        if persist {
            if let Some(kv) = &mut self.kv {
                kv.set(KV_SOUND_PACK, id);
            }
        }
        true
    }

    /// Get the currently active pack ID.
    pub fn current_pack_id(&self) -> String {
        self.current_pack().id.clone()
    }

    /// List all registered sound packs.
    pub fn list_packs(&self) -> Vec<SoundPackInfo> {
        let current = self.current_pack().id.clone();
        self.packs
            .values()
            .map(|pack| SoundPackInfo {
                id: pack.id.clone(),
                name: pack.name.clone(),
                active: pack.id == current,
                builtin: pack.builtin,
            })
            .collect()
    }

    /// Dispose the attention host.
    pub fn dispose(&mut self) {
        if self.disposed {
            return;
        }
        self.disposed = true;
    }
}

fn clamp_volume(volume: f64) -> f64 {
    if !volume.is_finite() {
        return 0.0;
    }
    volume.clamp(0.0, 1.0)
}

fn normalize_text(input: &str, fallback: &str, limit: usize) -> String {
    let stripped = strip_ansi(input);
    let cleaned: String = stripped
        .chars()
        .map(|c| {
            if c == '\r' || c == '\n' || c == '\t' {
                ' '
            } else if c.is_control() {
                '\0'
            } else {
                c
            }
        })
        .filter(|c| *c != '\0')
        .collect();
    let trimmed = cleaned.trim();
    let text = if trimmed.is_empty() {
        fallback
    } else {
        trimmed
    };
    text.chars().take(limit).collect()
}

fn strip_ansi(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            if let Some(&next) = chars.peek() {
                if next == '[' {
                    chars.next();
                    while let Some(ch) = chars.next() {
                        if ch.is_alphabetic() {
                            break;
                        }
                    }
                    continue;
                } else if next == ']' {
                    chars.next();
                    while let Some(ch) = chars.next() {
                        if ch == '\x07' {
                            break;
                        }
                    }
                    continue;
                }
            }
        }
        result.push(c);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clamp_volume() {
        assert_eq!(clamp_volume(0.5), 0.5);
        assert_eq!(clamp_volume(-1.0), 0.0);
        assert_eq!(clamp_volume(2.0), 1.0);
        assert_eq!(clamp_volume(f64::NAN), 0.0);
    }

    #[test]
    fn test_normalize_text_basic() {
        assert_eq!(normalize_text("hello", "", 240), "hello");
    }

    #[test]
    fn test_normalize_text_newlines() {
        assert_eq!(normalize_text("hello\nworld", "", 240), "hello world");
    }

    #[test]
    fn test_normalize_text_limit() {
        let result = normalize_text("abcdef", "", 3);
        assert_eq!(result, "abc");
    }

    #[test]
    fn test_normalize_text_fallback() {
        assert_eq!(normalize_text("", "fallback", 240), "fallback");
    }

    #[test]
    fn test_normalize_text_strips_ansi() {
        assert_eq!(normalize_text("\x1b[31mhello\x1b[0m", "", 240), "hello");
    }

    #[test]
    fn test_focus_skip_always() {
        let config = AttentionConfig::default();
        let mut attention = TuiAttention::new(config, None);
        let result = attention.focus_skip(AttentionWhen::Always, FocusState::Focused);
        assert_eq!(result, None);
    }

    #[test]
    fn test_focus_skip_blurred_when_focused() {
        let config = AttentionConfig::default();
        let mut attention = TuiAttention::new(config, None);
        let result = attention.focus_skip(AttentionWhen::Blurred, FocusState::Focused);
        assert_eq!(result, Some(NotifySkipReason::Focused));
    }

    #[test]
    fn test_focus_skip_focused_when_blurred() {
        let config = AttentionConfig::default();
        let mut attention = TuiAttention::new(config, None);
        let result = attention.focus_skip(AttentionWhen::Focused, FocusState::Blurred);
        assert_eq!(result, Some(NotifySkipReason::Blurred));
    }

    #[test]
    fn test_notify_disabled() {
        let mut config = AttentionConfig::default();
        config.enabled = false;
        let mut attention = TuiAttention::new(config, None);
        let input = NotifyInput {
            message: "test".to_string(),
            title: None,
            notification: NotifyConfig::Default,
            sound: SoundConfig::Disabled,
        };
        let result = attention.notify(&input);
        assert!(!result.ok);
        assert_eq!(result.skipped, Some(NotifySkipReason::AttentionDisabled));
    }

    #[test]
    fn test_notify_empty_message() {
        let config = AttentionConfig::default();
        let mut attention = TuiAttention::new(config, None);
        let input = NotifyInput {
            message: "".to_string(),
            title: None,
            notification: NotifyConfig::Default,
            sound: SoundConfig::Disabled,
        };
        let result = attention.notify(&input);
        assert!(!result.ok);
        assert_eq!(result.skipped, Some(NotifySkipReason::EmptyMessage));
    }

    #[test]
    fn test_notify_destroyed() {
        let config = AttentionConfig::default();
        let mut attention = TuiAttention::new(config, None);
        attention.mark_destroyed();
        let input = NotifyInput {
            message: "test".to_string(),
            title: None,
            notification: NotifyConfig::Default,
            sound: SoundConfig::Disabled,
        };
        let result = attention.notify(&input);
        assert!(!result.ok);
        assert_eq!(result.skipped, Some(NotifySkipReason::RendererDestroyed));
    }

    #[test]
    fn test_sound_name_from_str() {
        assert_eq!(SoundName::from_str("default"), Some(SoundName::Default));
        assert_eq!(SoundName::from_str("error"), Some(SoundName::Error));
        assert_eq!(SoundName::from_str("invalid"), None);
    }

    #[test]
    fn test_builtin_pack() {
        let pack = builtin_pack();
        assert_eq!(pack.id, DEFAULT_PACK_ID);
        assert!(pack.builtin);
        assert_eq!(pack.sounds.len(), 6);
    }

    #[test]
    fn test_list_packs() {
        let config = AttentionConfig::default();
        let attention = TuiAttention::new(config, None);
        let packs = attention.list_packs();
        assert_eq!(packs.len(), 1);
        assert!(packs[0].builtin);
        assert!(packs[0].active);
    }

    #[test]
    fn test_activate_pack() {
        let config = AttentionConfig::default();
        let mut attention = TuiAttention::new(config, None);
        assert!(attention.activate_pack(DEFAULT_PACK_ID, false));
        assert!(!attention.activate_pack("nonexistent", false));
    }

    #[test]
    fn test_dispose() {
        let config = AttentionConfig::default();
        let mut attention = TuiAttention::new(config, None);
        attention.dispose();
        attention.dispose();
    }

    #[test]
    fn test_strip_ansi() {
        assert_eq!(strip_ansi("\x1b[31mred\x1b[0m"), "red");
        assert_eq!(strip_ansi("\x1b]52;c;aGVsbG8=\x07"), "");
        assert_eq!(strip_ansi("plain"), "plain");
    }
}
