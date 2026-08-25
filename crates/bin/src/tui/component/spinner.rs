//! Spinner component — animated loading indicator.
//! Ported from tui/src/ui/spinner.ts and tui/src/component/spinner.tsx.
//!
//! The TS spinner has two layers:
//! 1. `spinner.ts` — Knight Rider scanner animation with gradient trail.
//! 2. `component/spinner.tsx` — simple braille dot spinner (10 frames, 80ms).
//!
//! This module implements the simple braille-dot spinner which is what the
//! TUI actually renders in places like `DialogPrompt`.

use std::time::{Duration, Instant};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::{Frame, layout::Rect};
use crate::tui::theme::Theme;

/// 10-frame braille-dot animation as used by the opencode TUI.
pub const SPINNER_FRAMES: &[&str] = &[
    "⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏",
];

/// Default frame interval: 80ms — matches the TS `80ms` tick.
pub const SPINNER_INTERVAL: Duration = Duration::from_millis(80);

/// Spinner interaction phase — mirrors claude-code-best's SpinnerMode.
/// Different modes map to different colors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpinnerMode {
    /// Model is thinking/planning (no output yet).
    Thinking,
    /// Request sent, waiting for first packet.
    Requesting,
    /// Model is streaming output.
    Responding,
    /// A tool is executing.
    ToolUse,
    /// Waiting for user to approve tool input/permissions.
    ToolInput,
}

impl Default for SpinnerMode {
    fn default() -> Self {
        Self::Thinking
    }
}

impl SpinnerMode {
    /// Pick a color for this mode (matches claude-code-best's palette).
    pub fn color(self, theme: &Theme) -> ratatui::style::Color {
        match self {
            Self::Thinking => theme.text_muted,
            Self::Requesting => theme.accent,
            Self::Responding => theme.primary,
            Self::ToolUse => theme.warning,
            Self::ToolInput => theme.secondary,
        }
    }

    /// The fallback label when no random verb is picked.
    pub fn fallback_label(self) -> &'static str {
        match self {
            Self::Thinking => "Thinking…",
            Self::Requesting => "Requesting…",
            Self::Responding => "Responding…",
            Self::ToolUse => "Running tool…",
            Self::ToolInput => "Waiting for input…",
        }
    }
}

/// Random spinner verbs (English) — mirrors claude-code-best's SPINNER_VERBS.
pub const SPINNER_VERBS_EN: &[&str] = &[
    "Accomplishing", "Cogitating", "Cooking", "Crafting", "Computing",
    "Contemplating", "Crunching", "Pondering", "Processing", "Reasoning",
    "Deliberating", "Synthesizing", "Formulating", "Architecting",
    "Bootstrapping", "Brewing", "Calculating", "Channeling",
    "Clauding", "Coalescing", "Composing", "Concocting", "Considering",
    "Converting", "Corralling", "Crystallizing", "Cultivating",
    "Deciphering", "Designing", "Deducing", "Developing", "Discovering",
];

/// Random spinner verbs (Chinese).
pub const SPINNER_VERBS_ZH: &[&str] = &[
    "正在处理", "正在思考", "正在计算", "正在构建", "正在分析",
    "正在优化", "正在整理", "正在推理", "正在生成", "正在解析",
    "正在搜索", "正在编排", "正在综合", "正在设计", "正在验证",
    "正在编译", "正在组织", "正在推导", "正在规划", "正在编写",
    "正在探索", "正在比对", "正在归纳", "正在提炼", "正在优化",
    "正在处理", "正在推导", "正在分析", "正在规划", "正在实现",
    "正在调试", "正在优化",
];

/// Get the verb list for the current locale.
fn verbs() -> &'static [&'static str] {
    // Check locale — if zh, use Chinese verbs.
    for key in &["LC_ALL", "LC_MESSAGES", "LANG"] {
        if let Ok(val) = std::env::var(key) {
            if val.to_lowercase().starts_with("zh") {
                return SPINNER_VERBS_ZH;
            }
        }
    }
    SPINNER_VERBS_EN
}

/// A simple spinner with a label and mode-aware color.
///
/// Mirrors the `<Spinner mode={...}>` component from claude-code-best.
pub struct Spinner {
    frame: usize,
    pub label: String,
    pub mode: SpinnerMode,
    last_tick: Instant,
    /// Random verb index — picked once per turn, like claude-code-best.
    verb_index: usize,
}

impl Spinner {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            frame: 0,
            label: label.into(),
            mode: SpinnerMode::Thinking,
            last_tick: Instant::now(),
            verb_index: rand::random::<usize>() % verbs().len(),
        }
    }

    pub fn with_color(self, _color: ratatui::style::Color) -> Self {
        // Color is now derived from mode — kept for backwards compat.
        self
    }

    /// Set the current mode (changes color + label).
    pub fn set_mode(&mut self, mode: SpinnerMode) {
        self.mode = mode;
    }

    /// Pick a new random verb (call at the start of each turn).
    pub fn pick_new_verb(&mut self) {
        self.verb_index = rand::random::<usize>() % verbs().len();
    }

    /// The current verb (e.g. "Cogitating") — used as the spinner label.
    fn current_verb(&self) -> &'static str {
        verbs()[self.verb_index]
    }

    /// Advance the animation if enough time has elapsed.
    /// Returns `true` if the frame changed.
    pub fn tick(&mut self) -> bool {
        if self.last_tick.elapsed() < SPINNER_INTERVAL {
            return false;
        }
        self.frame = (self.frame + 1) % SPINNER_FRAMES.len();
        self.last_tick = Instant::now();
        true
    }

    /// Force-advance one frame regardless of elapsed time.
    pub fn step(&mut self) {
        self.frame = (self.frame + 1) % SPINNER_FRAMES.len();
    }

    pub fn reset(&mut self) {
        self.frame = 0;
        self.last_tick = Instant::now();
        self.pick_new_verb();
    }

    pub fn current_frame(&self) -> &'static str {
        SPINNER_FRAMES[self.frame]
    }

    /// The display label: random verb + "…" (like claude-code-best), or
    /// the fallback label for the current mode if no verb.
    /// Uses i18n for the fallback labels (locale-aware).
    pub fn display_label(&self) -> String {
        // Use the random verb for thinking/requesting/responding modes,
        // fallback for tool modes (which have specific labels).
        match self.mode {
            SpinnerMode::Thinking | SpinnerMode::Requesting | SpinnerMode::Responding => {
                format!("{}…", self.current_verb())
            }
            SpinnerMode::ToolUse => {
                crate::t!("tui.spinner.tool_use").to_string()
            }
            SpinnerMode::ToolInput => {
                crate::t!("tui.spinner.tool_input").to_string()
            }
        }
    }

    /// Build a styled `Line` for embedding in other widgets.
    pub fn line(&self, theme: &Theme) -> Line<'static> {
        let fg = self.mode.color(theme);
        Line::from(vec![
            Span::styled(self.current_frame().to_string(), Style::default().fg(fg)),
            Span::raw(" "),
            Span::styled(self.display_label(), Style::default().fg(fg)),
        ])
    }

    /// Render the spinner directly into a frame area.
    pub fn render(&self, f: &mut Frame, area: Rect, theme: &Theme) {
        let fg = self.mode.color(theme);
        let line = Line::from(vec![
            Span::styled(self.current_frame().to_string(), Style::default().fg(fg)),
            Span::raw(" "),
            Span::styled(self.display_label(), Style::default().fg(fg)),
        ]);
        f.render_widget(Paragraph::new(line), area);
    }
}

impl Default for Spinner {
    fn default() -> Self {
        Self::new("Working...")
    }
}

// ---------------------------------------------------------------------------
// Knight Rider scanner — ported from spinner.ts
// ---------------------------------------------------------------------------

/// Style of the scanner shapes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KnightRiderStyle {
    Blocks,
    Diamonds,
}

/// Options for the Knight Rider scanner animation.
pub struct KnightRiderOptions {
    pub width: usize,
    pub style: KnightRiderStyle,
    pub hold_start: usize,
    pub hold_end: usize,
}

impl Default for KnightRiderOptions {
    fn default() -> Self {
        Self {
            width: 8,
            style: KnightRiderStyle::Diamonds,
            hold_start: 30,
            hold_end: 9,
        }
    }
}

/// Generate frame strings for a Knight Rider scanner.
///
/// Ported from `createFrames()` in `spinner.ts`. Produces a vector of
/// frame strings where each character is an active or inactive glyph.
pub fn create_knight_rider_frames(opts: &KnightRiderOptions) -> Vec<String> {
    let width = opts.width;
    let total_frames = width + opts.hold_end + (width.saturating_sub(1)) + opts.hold_start;
    let trail_length = 6usize;

    (0..total_frames)
        .map(|frame_index| {
            (0..width)
                .map(|char_index| {
                    let idx = calculate_color_index(frame_index, char_index, width, trail_length, opts);
                    match opts.style {
                        KnightRiderStyle::Diamonds => {
                            let shapes = ["⬥", "◆", "⬩", "⬪"];
                            if idx >= 0 && (idx as usize) < trail_length {
                                shapes[(idx as usize).min(shapes.len() - 1)]
                            } else {
                                "·"
                            }
                        }
                        KnightRiderStyle::Blocks => {
                            if idx >= 0 && (idx as usize) < trail_length {
                                "■"
                            } else {
                                "⬝"
                            }
                        }
                    }
                })
                .collect::<String>()
        })
        .collect()
}

/// Compute the color/trail index for a given frame and character position.
///
/// Ported from `calculateColorIndex()` + `getScannerState()` in `spinner.ts`.
fn calculate_color_index(frame_index: usize, char_index: usize, total_chars: usize, trail_length: usize, opts: &KnightRiderOptions) -> i32 {
    let width = total_chars;
    let forward_frames = width;
    let hold_end = opts.hold_end;
    let backward_frames = width.saturating_sub(1);

    let (active_position, is_holding, hold_progress, is_moving_forward): (usize, bool, usize, bool) = if frame_index < forward_frames {
        (frame_index, false, 0, true)
    } else if frame_index < forward_frames + hold_end {
        (total_chars - 1, true, frame_index - forward_frames, true)
    } else if frame_index < forward_frames + hold_end + backward_frames {
        let backward_index = frame_index - forward_frames - hold_end;
        (total_chars.saturating_sub(2).saturating_sub(backward_index), false, 0, false)
    } else {
        (0, true, frame_index - forward_frames - hold_end - backward_frames, false)
    };

    let directional_distance = if is_moving_forward {
        active_position as i32 - char_index as i32
    } else {
        char_index as i32 - active_position as i32
    };

    if is_holding {
        return directional_distance + hold_progress as i32;
    }

    if directional_distance > 0 && directional_distance < trail_length as i32 {
        return directional_distance;
    }

    if directional_distance == 0 {
        return 0;
    }

    -1
}
