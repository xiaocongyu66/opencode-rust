//! Click registry — tracks clickable screen regions for mouse interaction.
//!
//! Each render frame clears the registry, components register their clickable
//! areas (with a callback id), and mouse events look up the region under the
//! cursor to trigger the corresponding action.
//!
//! This is a simple spatial index: regions are stored in a Vec and searched
//! linearly (fine for the ~20 clickable elements typical in a TUI frame).

use ratatui::layout::Rect;

/// An identifier for a clickable action. Stored as a string so components
/// can define their own action namespaces (e.g. "model:select", "session:switch:ses_xxx").
#[derive(Debug, Clone)]
pub struct ClickAction(pub String);

/// A registered clickable region.
#[derive(Debug, Clone)]
pub struct ClickRegion {
    pub rect: Rect,
    pub action: ClickAction,
    /// Optional tooltip/hover text shown when the cursor is over this region.
    pub hover: Option<String>,
}

/// Registry of clickable regions for the current frame.
#[derive(Debug, Default)]
pub struct ClickRegistry {
    regions: Vec<ClickRegion>,
    /// Last hovered region's action, for hover-highlight rendering.
    pub hovered: Option<String>,
}

impl ClickRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Clear all registered regions. Called at the start of each render frame.
    pub fn clear(&mut self) {
        self.regions.clear();
        // Keep `hovered` — it's updated in handle_mouse_move.
    }

    /// Register a clickable region. Returns nothing; components call this
    /// during render to declare "this rect is clickable and does X".
    pub fn register(
        &mut self,
        rect: Rect,
        action: impl Into<String>,
        hover: Option<String>,
    ) {
        self.regions.push(ClickRegion {
            rect,
            action: ClickAction(action.into()),
            hover,
        });
    }

    /// Find the region containing the given (column, row).
    pub fn hit_test(&self, col: u16, row: u16) -> Option<&ClickRegion> {
        self.regions.iter().find(|r| {
            col >= r.rect.x
                && col < r.rect.x + r.rect.width
                && row >= r.rect.y
                && row < r.rect.y + r.rect.height
        })
    }

    /// Update the hovered region for mouse-move events.
    /// Returns true if the hover changed (so the caller can trigger a redraw).
    pub fn update_hover(&mut self, col: u16, row: u16) -> bool {
        let new_hover = self
            .hit_test(col, row)
            .map(|r| r.action.0.clone());
        if new_hover == self.hovered {
            false
        } else {
            self.hovered = new_hover;
            true
        }
    }

    /// All registered regions (for debugging or custom hit-testing).
    pub fn regions(&self) -> &[ClickRegion] {
        &self.regions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect(x: u16, y: u16, w: u16, h: u16) -> Rect {
        Rect { x, y, width: w, height: h }
    }

    #[test]
    fn register_and_hit_test() {
        let mut reg = ClickRegistry::new();
        reg.register(rect(10, 5, 20, 3), "model:select", None);
        let hit = reg.hit_test(15, 6);
        assert!(hit.is_some());
        assert_eq!(hit.unwrap().action.0, "model:select");
    }

    #[test]
    fn miss_returns_none() {
        let mut reg = ClickRegistry::new();
        reg.register(rect(10, 5, 20, 3), "model:select", None);
        assert!(reg.hit_test(0, 0).is_none());
        assert!(reg.hit_test(10, 5).is_some()); // top-left corner
        assert!(reg.hit_test(29, 7).is_some()); // bottom-right corner
        assert!(reg.hit_test(30, 5).is_none()); // right edge + 1
    }

    #[test]
    fn clear_removes_all_regions() {
        let mut reg = ClickRegistry::new();
        reg.register(rect(0, 0, 10, 10), "a", None);
        reg.register(rect(0, 0, 10, 10), "b", None);
        assert_eq!(reg.regions().len(), 2);
        reg.clear();
        assert_eq!(reg.regions().len(), 0);
    }

    #[test]
    fn update_hover_detects_change() {
        let mut reg = ClickRegistry::new();
        reg.register(rect(0, 0, 5, 5), "region_a", None);
        reg.register(rect(10, 0, 5, 5), "region_b", None);

        // Hover over region A.
        assert!(reg.update_hover(2, 2));
        assert_eq!(reg.hovered.as_deref(), Some("region_a"));

        // Move within A — no change.
        assert!(!reg.update_hover(3, 3));

        // Move to B — change.
        assert!(reg.update_hover(12, 2));
        assert_eq!(reg.hovered.as_deref(), Some("region_b"));

        // Move to empty space — change to None.
        assert!(reg.update_hover(7, 2));
        assert!(reg.hovered.is_none());
    }

    #[test]
    fn multiple_regions_same_point_last_wins() {
        let mut reg = ClickRegistry::new();
        reg.register(rect(0, 0, 10, 10), "first", None);
        reg.register(rect(0, 0, 10, 10), "second", None);
        let hit = reg.hit_test(5, 5).unwrap();
        // find() returns the first match, so "first" wins.
        assert_eq!(hit.action.0, "first");
    }
}
