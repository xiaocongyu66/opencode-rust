//! ScrollView — unified scroll/pan/zoom state for the message list.
//!
//! Consolidates what used to be scattered App fields (`scroll`, `auto_scroll`,
//! `drag_last_row`, `drag_on_scrollbar`, `last_max_scroll`, `scroll_accel`,
//! `scroll_subpixel`, `show_scrollbar`, `scrollbar_area`) into one component.
//! All keyboard, wheel, and drag input flows through here, and `render()` draws
//! both the content paragraph and the optional scrollbar.

use ratatui::layout::Rect;
use ratatui::text::Line;
use ratatui::widgets::{Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap};
use ratatui::Frame;

use crate::tui::util::scroll::{MacOSScrollAccel, ScrollAcceleration};

/// Natural-scroll scroll view for a list of rendered lines.
pub struct ScrollView {
    /// Integer scroll offset (lines from the top). Cached at f64 precision via
    /// `scroll_f` so sub-line wheel accumulation isn't lost.
    pub scroll: usize,
    /// Sub-line remainder accumulated by accelerated scrolling.
    pub scroll_subpixel: f64,
    /// When true, the view pins to the bottom and follows new content.
    pub auto_follow: bool,
    /// Last-rendered max scroll offset (total_lines - visible_height).
    pub max_scroll: usize,
    /// macOS-style acceleration for mouse-wheel scrolling.
    pub accel: MacOSScrollAccel,
    /// Whether the vertical scrollbar is drawn.
    pub show_scrollbar: bool,
    /// Last-rendered scrollbar track area (for hit-testing drags).
    pub scrollbar_area: Option<Rect>,
    /// Last row seen on mouse Down/Drag — basis for natural-scroll drag.
    pub drag_last_row: Option<u16>,
    /// True when the current drag started on the scrollbar track.
    pub drag_on_scrollbar: bool,
    /// Per-message starting line offsets, populated during render. Used by
    /// `scroll_to_next_message` for message-boundary jumps.
    pub message_offsets: Vec<usize>,
}

impl Default for ScrollView {
    fn default() -> Self {
        Self::new()
    }
}

impl ScrollView {
    pub fn new() -> Self {
        Self {
            scroll: 0,
            scroll_subpixel: 0.0,
            auto_follow: true,
            max_scroll: 0,
            accel: MacOSScrollAccel::new(),
            show_scrollbar: false,
            scrollbar_area: None,
            drag_last_row: None,
            drag_on_scrollbar: false,
            message_offsets: Vec::new(),
        }
    }

    /// Whether the view is pinned to the bottom (within 1 line of max_scroll).
    pub fn is_at_bottom(&self) -> bool {
        self.scroll >= self.max_scroll.saturating_sub(1)
    }

    /// Follow new content only if already at the bottom (sticky behavior).
    pub fn follow_if_at_bottom(&mut self) {
        if self.is_at_bottom() {
            self.auto_follow = true;
        }
    }

    /// Pin to the bottom unconditionally (e.g. on user submit / resume).
    /// Resets scroll so the next render pins to max_scroll regardless of
    /// where the user previously scrolled.
    pub fn scroll_to_bottom(&mut self) {
        self.auto_follow = true;
        self.scroll = 0;
        self.scroll_subpixel = 0.0;
    }

    /// Pin to the top.
    pub fn scroll_to_top(&mut self) {
        self.auto_follow = false;
        self.scroll = 0;
        self.scroll_subpixel = 0.0;
    }

    /// Called after content changes (new message, text delta, etc.). If the
    /// view was already at the bottom, follow the new content. Otherwise
    /// keep the user's scroll position. This is the tui4j-style "stay at
    /// bottom if at bottom" behavior without a separate auto_follow flag
    /// — but we keep the flag for compatibility.
    pub fn on_content_changed(&mut self) {
        if self.is_at_bottom() {
            self.auto_follow = true;
        }
    }

    /// Jump to an absolute line offset, clamped to [0, max_scroll].
    pub fn scroll_to(&mut self, line: usize) {
        self.auto_follow = false;
        self.scroll = line.min(self.max_scroll);
        self.scroll_subpixel = 0.0;
    }

    /// Toggle the scrollbar visibility.
    pub fn toggle_scrollbar(&mut self) {
        self.show_scrollbar = !self.show_scrollbar;
    }

    // --- input handlers ----------------------------------------------------

    /// Mouse-wheel up: feed an impulse and apply accumulated velocity.
    pub fn on_wheel_up(&mut self) {
        self.auto_follow = false;
        self.accel.feed(3.0);
        let step = self.accel.tick();
        self.scroll_subpixel -= step;
        let lines = self.scroll_subpixel.floor() as i64;
        if lines != 0 {
            if lines < 0 {
                self.scroll = self.scroll.saturating_sub(lines.unsigned_abs() as usize);
            } else {
                self.scroll = self.scroll.saturating_add(lines as usize);
            }
            self.scroll_subpixel -= lines as f64;
            self.scroll = self.scroll.min(self.max_scroll);
        }
    }

    /// Mouse-wheel down: feed an impulse and apply accumulated velocity.
    pub fn on_wheel_down(&mut self) {
        self.auto_follow = false;
        self.accel.feed(3.0);
        let step = self.accel.tick();
        self.scroll_subpixel += step;
        let lines = self.scroll_subpixel.floor() as i64;
        if lines != 0 {
            self.scroll = self.scroll.saturating_add(lines as usize);
            self.scroll_subpixel -= lines as f64;
            self.scroll = self.scroll.min(self.max_scroll);
        }
    }

    /// Record the start of a drag gesture. `row` is the terminal row; `on_bar`
    /// indicates whether the press landed on the scrollbar track.
    pub fn on_drag_start(&mut self, row: u16, on_bar: bool) {
        self.drag_on_scrollbar = on_bar;
        self.drag_last_row = Some(row);
    }

    /// Update an in-progress drag. `row` is the current terminal row.
    pub fn on_drag(&mut self, row: u16) {
        if self.drag_on_scrollbar {
            // Thumb-scrub: map cursor row within the track to a position.
            if let Some(r) = self.scrollbar_area {
                if r.height > 0 {
                    self.auto_follow = false;
                    let rel = (row as i64 - r.y as i64).max(0) as usize;
                    let ratio = rel as f64 / r.height as f64;
                    self.scroll = (ratio * self.max_scroll as f64) as usize;
                    self.scroll = self.scroll.min(self.max_scroll);
                }
            }
        } else if let Some(prev_row) = self.drag_last_row {
            // Natural scroll: content follows the finger.
            let delta = row as i32 - prev_row as i32;
            if delta != 0 {
                self.auto_follow = false;
                if delta > 0 {
                    self.scroll = self.scroll.saturating_sub(delta.abs() as usize);
                } else {
                    self.scroll = self.scroll.saturating_add(delta.abs() as usize);
                }
                self.scroll = self.scroll.min(self.max_scroll);
            }
        }
        self.drag_last_row = Some(row);
    }

    /// End a drag gesture.
    pub fn on_drag_end(&mut self) {
        self.drag_last_row = None;
        self.drag_on_scrollbar = false;
    }

    /// Apply a discrete line delta (used by keyboard arrows / page keys).
    /// Positive `lines` scrolls down (toward newer messages).
    pub fn on_line_delta(&mut self, lines: i64) {
        self.auto_follow = false;
        if lines > 0 {
            self.scroll = self.scroll.saturating_add(lines as usize).min(self.max_scroll);
        } else {
            self.scroll = self.scroll.saturating_sub(lines.unsigned_abs() as usize);
        }
        self.scroll_subpixel = 0.0;
    }

    /// Jump to the next or previous message boundary.
    /// `forward = true` → next (later) message, `false` → previous (earlier).
    pub fn scroll_to_next_message(&mut self, forward: bool) {
        if self.message_offsets.is_empty() {
            // No offsets recorded — fall back to a page jump.
            let page = self.max_scroll.max(1);
            self.on_line_delta(if forward { page as i64 } else { -(page as i64) });
            return;
        }
        let cur = self.scroll;
        let target = if forward {
            // First message boundary strictly after current position.
            self.message_offsets
                .iter()
                .find(|&&off| off > cur)
                .copied()
                .unwrap_or(self.max_scroll)
        } else {
            // Last message boundary strictly before current position.
            self.message_offsets
                .iter()
                .rev()
                .find(|&&off| off + 1 < cur)
                .copied()
                .unwrap_or(0)
        };
        self.scroll_to(target);
    }

    // --- rendering --------------------------------------------------------

    /// Render the content lines + optional scrollbar into `area`. Returns the
    /// resolved scroll offset used (useful for click-registering callers).
    pub fn render<'a>(
        &mut self,
        f: &mut Frame,
        area: Rect,
        lines: Vec<Line<'a>>,
        click_registry: &mut crate::tui::app::click_registry::ClickRegistry,
    ) -> usize {
        let total = lines.len();
        let visible = area.height as usize;
        let max_scroll = total.saturating_sub(visible);
        self.max_scroll = max_scroll;

        let scroll = if self.auto_follow {
            max_scroll
        } else {
            self.scroll.min(max_scroll)
        };

        f.render_widget(
            Paragraph::new(lines).scroll((scroll as u16, 0)).wrap(Wrap { trim: false }),
            area,
        );

        if self.show_scrollbar && max_scroll > 0 {
            let mut sb_state = ScrollbarState::new(max_scroll).position(scroll);
            f.render_stateful_widget(
                Scrollbar::new(ScrollbarOrientation::VerticalRight),
                area,
                &mut sb_state,
            );
            let track_area = Rect {
                x: area.right().saturating_sub(1),
                y: area.y,
                width: 1,
                height: area.height,
            };
            self.scrollbar_area = Some(track_area);
            click_registry.register(track_area, "scroll:bar", None);
        } else {
            self.scrollbar_area = None;
        }

        scroll
    }
}
