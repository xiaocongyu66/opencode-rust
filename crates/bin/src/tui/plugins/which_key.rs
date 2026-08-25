use crate::tui::plugins::builtins::BuiltinTuiPlugin;
use std::collections::HashMap;

pub const WHICH_KEY_ID: &str = "which-key";

pub const LAYER_PRIORITY: u32 = 900;
pub const KV_LAYOUT: &str = "which_key_layout";
pub const KV_PENDING_PREVIEW: &str = "which_key_pending_preview";

pub const COLUMN_GAP: usize = 4;
pub const TAB_GAP: usize = 3;
pub const MIN_TAB_GAP: usize = 1;
pub const TAB_CONTENT_GAP: usize = 1;
pub const MIN_COLUMN_WIDTH: usize = 28;
pub const MAX_COLUMN_WIDTH: usize = 44;
pub const PANEL_HEIGHT_RATIO: f64 = 0.3;
pub const MIN_PANEL_HEIGHT: usize = 8;
pub const MAX_PANEL_HEIGHT: usize = 16;
pub const PANEL_TOP_PADDING: usize = 1;
pub const FOOTER_HEIGHT: usize = 1;
pub const FOOTER_MARGIN: usize = 1;

pub const TOGGLE_COMMANDS: &[&str] = &[
    "which-key.toggle",
    "which-key.layout.toggle",
    "which-key.pending.toggle",
];

pub const SCROLL_COMMANDS: &[&str] = &[
    "which-key.scroll.up",
    "which-key.scroll.down",
    "which-key.page.up",
    "which-key.page.down",
    "which-key.home",
    "which-key.end",
];

pub const PANEL_COMMANDS: &[&str] = &[
    "which-key.group.previous",
    "which-key.group.next",
    "which-key.scroll.up",
    "which-key.scroll.down",
    "which-key.page.up",
    "which-key.page.down",
    "which-key.home",
    "which-key.end",
];

pub type Layout = &'static str;
pub const LAYOUT_DOCK: &str = "dock";
pub const LAYOUT_OVERLAY: &str = "overlay";

pub fn layout_from_str(value: Option<&str>) -> Layout {
    match value {
        Some("overlay") => LAYOUT_OVERLAY,
        _ => LAYOUT_DOCK,
    }
}

pub fn next_layout(current: Layout) -> Layout {
    if current == LAYOUT_DOCK { LAYOUT_OVERLAY } else { LAYOUT_DOCK }
}

pub struct WhichKeyPlugin;

impl WhichKeyPlugin {
    pub fn builtin() -> BuiltinTuiPlugin {
        BuiltinTuiPlugin::disabled(WHICH_KEY_ID).with_order(100)
    }

    pub fn id() -> &'static str { WHICH_KEY_ID }
}

#[derive(Clone, Debug)]
pub struct Entry {
    pub key: String,
    pub label: String,
    pub group: String,
    pub continues: bool,
}

#[derive(Clone, Debug)]
pub struct Group {
    pub label: String,
    pub entries: Vec<Entry>,
}

#[derive(Clone, Debug)]
pub enum HeaderItem {
    Tab { group: Group },
    Scroll,
}

pub enum Item {
    Entry(Entry),
    GroupHeader { label: String },
}

pub fn active_key_label(
    command_title: Option<&str>,
    binding_desc: Option<&str>,
    command_desc: Option<&str>,
    token_name: Option<&str>,
    display: Option<&str>,
    continues: bool,
) -> String {
    if continues {
        return token_name.or(display).unwrap_or("Unknown").to_string();
    }
    command_title
        .or(binding_desc)
        .or(command_desc)
        .unwrap_or("Unknown")
        .to_string()
}

pub fn active_key_group(
    command_category: Option<&str>,
    binding_group: Option<&str>,
    continues: bool,
) -> String {
    if continues { return "System".to_string() }
    command_category
        .or(binding_group)
        .unwrap_or("Unknown")
        .to_string()
}

pub fn active_key_entry(
    key: String,
    label: String,
    group: String,
    continues: bool,
) -> Entry {
    Entry {
        key,
        label: if continues { format!("+{}", label) } else { label },
        group,
        continues,
    }
}

pub fn grouped(entries: Vec<Entry>) -> Vec<Group> {
    let mut map: HashMap<String, Vec<Entry>> = HashMap::new();
    let mut order: Vec<String> = Vec::new();
    for entry in entries {
        if !map.contains_key(&entry.group) {
            order.push(entry.group.clone());
        }
        map.entry(entry.group.clone()).or_default().push(entry);
    }
    let mut groups: Vec<Group> = order
        .into_iter()
        .map(|label| {
            let mut items = map.remove(&label).unwrap_or_default();
            items.sort_by(|a, b| {
                (!b.continues as u8)
                    .cmp(&(!a.continues as u8))
                    .then_with(|| a.label.cmp(&b.label))
                    .then_with(|| a.key.cmp(&b.key))
            });
            Group { label, entries: items }
        })
        .collect();
    groups.sort_by(|a, b| a.label.cmp(&b.label));
    groups
}

pub fn panel_height(terminal_height: usize) -> usize {
    let raw = (terminal_height as f64 * PANEL_HEIGHT_RATIO) as usize;
    raw.max(MIN_PANEL_HEIGHT).min(MAX_PANEL_HEIGHT)
}

pub fn columns(content_width: usize) -> usize {
    let denominator = MAX_COLUMN_WIDTH + COLUMN_GAP;
    if denominator == 0 { return 1 }
    let count = (content_width + COLUMN_GAP) / denominator;
    count.max(1).min(3)
}

pub fn column_width(content_width: usize, columns: usize) -> usize {
    if columns == 0 { return 1 }
    let total_gap = (columns - 1) * COLUMN_GAP;
    let width = (content_width.saturating_sub(total_gap)) / columns;
    width.max(1).min(MAX_COLUMN_WIDTH)
}

pub fn rows(
    panel_height: usize,
    header_visible: bool,
    tabs_visible: bool,
    footer_visible: bool,
) -> usize {
    let mut r = panel_height
        .saturating_sub(PANEL_TOP_PADDING)
        .saturating_sub(if header_visible { 1 } else { 0 })
        .saturating_sub(if tabs_visible { TAB_CONTENT_GAP } else { 0 })
        .saturating_sub(if footer_visible { FOOTER_MARGIN + FOOTER_HEIGHT } else { 0 });
    if r == 0 { r = 1 }
    r
}

pub fn page_size(rows: usize, columns: usize) -> usize {
    rows * columns
}

pub fn clamp_offset(offset: usize, max_offset: usize) -> usize {
    offset.min(max_offset)
}

pub fn scroll_offset(offset: usize, delta: i32, max_offset: usize) -> usize {
    let next = offset as i32 + delta;
    if next < 0 { 0 } else { next as usize }.min(max_offset)
}

pub fn move_group(groups: &[Group], current: Option<&str>, delta: i32) -> Option<String> {
    if groups.is_empty() { return None }
    let index = match current {
        Some(label) => groups.iter().position(|g| g.label == label).unwrap_or(0),
        None => 0,
    };
    let len = groups.len() as i32;
    let next = ((index as i32 + delta + len) % len) as usize;
    Some(groups[next].label.clone())
}

pub fn build_items(active_entries: &[Entry], groups: &[Group], pending_mode: bool) -> Vec<Item> {
    if !pending_mode {
        return active_entries
            .iter()
            .map(|e| Item::Entry(e.clone()))
            .collect();
    }
    let mut items = Vec::new();
    for group in groups {
        items.push(Item::GroupHeader { label: group.label.clone() });
        for entry in &group.entries {
            items.push(Item::Entry(entry.clone()));
        }
    }
    items
}

pub fn paginate(items: &[Item], offset: usize, rows: usize, columns: usize) -> Vec<Vec<&Item>> {
    let mut result = Vec::new();
    let mut index = offset;
    for _ in 0..columns {
        if index >= items.len() { break }
        let mut column = Vec::new();
        for _ in 0..rows {
            if index >= items.len() { break }
            column.push(&items[index]);
            index += 1;
        }
        result.push(column);
    }
    result
}

pub fn max_offset(items_len: usize, page_size: usize) -> usize {
    items_len.saturating_sub(page_size)
}

pub fn tab_gap(header_items: &[HeaderItem], content_width: usize) -> usize {
    let item_count = header_items.len();
    if item_count <= 1 { return 0 }
    let item_width: usize = header_items
        .iter()
        .map(|item| match item {
            HeaderItem::Tab { group } => group.label.len() + 2,
            HeaderItem::Scroll => 3,
        })
        .sum();
    let gap = (content_width.saturating_sub(item_width)) / (item_count - 1);
    gap.max(MIN_TAB_GAP).min(TAB_GAP)
}

pub fn skin() -> Skin {
    Skin {
        panel: "backgroundMenu",
        text: "text",
        muted: "textMuted",
        subtle: "borderSubtle",
        key: "warning",
        accent: "primary",
        tab: "primary",
        tab_text: "selectedListItemText",
    }
}

pub struct Skin {
    pub panel: &'static str,
    pub text: &'static str,
    pub muted: &'static str,
    pub subtle: &'static str,
    pub key: &'static str,
    pub accent: &'static str,
    pub tab: &'static str,
    pub tab_text: &'static str,
}

pub fn ink(theme_field: &str) -> &'static str {
    match theme_field {
        "backgroundMenu" => "#1c1c1c",
        "text" => "#f0f0f0",
        "textMuted" => "#a5a5a5",
        "borderSubtle" => "#6f6f6f",
        "warning" => "#ffd75f",
        "primary" => "#5f87ff",
        "selectedListItemText" => "#ffffff",
        _ => "#ffffff",
    }
}

pub fn home_hint_text(trigger: &str) -> String {
    if trigger.is_empty() {
        format!("Show keyboard shortcuts with which-key.toggle")
    } else {
        format!("Show keyboard shortcuts with {}", trigger)
    }
}
