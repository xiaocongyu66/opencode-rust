use crate::tui::plugins::builtins::BuiltinTuiPlugin;
use std::collections::{HashMap, HashSet};

pub const ROUTE: &str = "diff";
pub const MIN_SPLIT_WIDTH: usize = 100;
pub const FILE_TREE_WIDTH: usize = 32;
pub const PLAIN_TEXT_FILETYPE: &str = "opencode-plain-text";
pub const VCS_DIFF_CONTEXT_LINES: usize = 12;
pub const KV_SHOW_FILE_TREE: &str = "diff_viewer_show_file_tree";
pub const KV_SINGLE_PATCH: &str = "diff_viewer_single_patch";
pub const KV_VIEW: &str = "diff_viewer_view";

pub type DiffMode = &'static str;
pub const DIFF_MODE_GIT: &str = "git";
pub const DIFF_MODE_BRANCH: &str = "branch";
pub const DIFF_MODE_LAST_TURN: &str = "last-turn";

pub type DiffViewerFocus = &'static str;
pub const FOCUS_PATCHES: &str = "patches";
pub const FOCUS_FILES: &str = "files";

pub type DiffView = &'static str;
pub const VIEW_SPLIT: &str = "split";
pub const VIEW_UNIFIED: &str = "unified";

pub struct DiffViewerPlugin;

impl DiffViewerPlugin {
    pub fn builtin() -> BuiltinTuiPlugin {
        BuiltinTuiPlugin::new("diff-viewer").with_order(100)
    }

    pub fn id() -> &'static str { "diff-viewer" }

    pub fn route() -> &'static str { ROUTE }
}

#[derive(Clone, Debug)]
pub struct DiffFile {
    pub file: String,
    pub patch: Option<String>,
    pub additions: u32,
    pub deletions: u32,
    pub status: DiffFileStatus,
}

#[derive(Clone, Debug, PartialEq)]
pub enum DiffFileStatus {
    Added,
    Deleted,
    Modified,
}

impl DiffFileStatus {
    pub fn from_str(s: &str) -> Self {
        match s {
            "added" => Self::Added,
            "deleted" => Self::Deleted,
            _ => Self::Modified,
        }
    }
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Added => "added",
            Self::Deleted => "deleted",
            Self::Modified => "modified",
        }
    }
    pub fn marker(&self) -> char {
        match self {
            Self::Added => 'A',
            Self::Deleted => 'D',
            Self::Modified => 'M',
        }
    }
}

pub fn normalize_diffs(diffs: &[RawDiff]) -> Vec<DiffFile> {
    diffs
        .iter()
        .filter_map(|d| {
            if d.file.is_empty() { return None }
            Some(DiffFile {
                file: d.file.clone(),
                patch: d.patch.clone(),
                additions: d.additions,
                deletions: d.deletions,
                status: DiffFileStatus::from_str(d.status.as_deref().unwrap_or("modified")),
            })
        })
        .collect()
}

pub struct RawDiff {
    pub file: String,
    pub patch: Option<String>,
    pub additions: u32,
    pub deletions: u32,
    pub status: Option<String>,
}

pub fn diff_source_label(mode: &str) -> &'static str {
    match mode {
        DIFF_MODE_LAST_TURN => "last turn",
        DIFF_MODE_BRANCH => "main branch",
        _ => "working tree",
    }
}

pub fn filetype(input: Option<&str>) -> &str {
    let input = match input { Some(s) => s, None => return "none" };
    let ext = input.rsplit('.').next().unwrap_or("");
    match ext {
        "ts" | "tsx" | "js" | "jsx" => "typescript",
        "rs" => "rust",
        "py" => "python",
        "go" => "go",
        "json" => "json",
        "md" => "markdown",
        "css" => "css",
        "html" => "html",
        "toml" => "toml",
        "yaml" | "yml" => "yaml",
        _ => "none",
    }
}

pub fn stored_view(value: Option<&str>) -> Option<&'static str> {
    match value {
        Some("split") => Some(VIEW_SPLIT),
        Some("unified") => Some(VIEW_UNIFIED),
        _ => None,
    }
}

#[derive(Clone, Debug)]
pub struct FileTreeItem {
    pub file: String,
    pub status: Option<DiffFileStatus>,
}

#[derive(Clone, Debug)]
pub struct FileTreeNode {
    pub id: usize,
    pub name: String,
    pub parent: Option<usize>,
    pub children: Vec<usize>,
    pub depth: usize,
    pub kind: FileTreeKind,
    pub file_index: Option<usize>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum FileTreeKind {
    Directory,
    File,
}

#[derive(Clone, Debug)]
pub struct FileTree {
    pub roots: Vec<usize>,
    pub nodes: Vec<FileTreeNode>,
}

#[derive(Clone, Debug)]
pub struct FileTreeRow {
    pub id: usize,
    pub depth: usize,
    pub kind: FileTreeKind,
    pub name: String,
    pub file_index: Option<usize>,
}

pub fn build_file_tree(files: &[FileTreeItem]) -> FileTree {
    let mut roots: Vec<usize> = Vec::new();
    let mut nodes: Vec<FileTreeNode> = Vec::new();
    let mut directory_by_path: HashMap<String, usize> = HashMap::new();

    for (file_index, file) in files.iter().enumerate() {
        let segments: Vec<&str> = file.file.split('/').filter(|s| !s.is_empty()).collect();
        if segments.is_empty() { continue }

        let mut current_id: Option<usize> = None;
        let mut current_path = String::new();
        let mut current_depth = 0usize;

        for segment in &segments[..segments.len() - 1] {
            let dir_path = if current_path.is_empty() {
                segment.to_string()
            } else {
                format!("{}/{}", current_path, segment)
            };

            if let Some(&existing) = directory_by_path.get(&dir_path) {
                current_id = Some(existing);
                current_path = dir_path;
                current_depth += 1;
                continue;
            }

            let id = add_file_tree_node(&mut nodes, &mut roots, FileTreeNode {
                name: segment.to_string(),
                parent: current_id,
                depth: current_depth,
                kind: FileTreeKind::Directory,
                children: Vec::new(),
                file_index: None,
                id: 0,
            });
            directory_by_path.insert(dir_path.clone(), id);
            current_id = Some(id);
            current_path = dir_path;
            current_depth += 1;
        }

        add_file_tree_node(&mut nodes, &mut roots, FileTreeNode {
            name: segments[segments.len() - 1].to_string(),
            parent: current_id,
            depth: current_depth,
            kind: FileTreeKind::File,
            children: Vec::new(),
            file_index: Some(file_index),
            id: 0,
        });
    }

    let mut tree = FileTree { roots, nodes };
    tree.roots.sort_by(|a, b| compare_file_tree_nodes(&tree, *a, *b));
    for node in &mut tree.nodes {
        node.children.sort_by(|a, b| compare_file_tree_nodes(&tree, *a, *b));
    }
    tree
}

fn add_file_tree_node(
    nodes: &mut Vec<FileTreeNode>,
    roots: &mut Vec<usize>,
    mut input: FileTreeNode,
) -> usize {
    let id = nodes.len();
    input.id = id;
    let parent = input.parent;
    nodes.push(input);
    match parent {
        None => roots.push(id),
        Some(p) => {
            if let Some(node) = nodes.get_mut(p) {
                node.children.push(id);
            }
        }
    }
    id
}

pub fn compare_file_tree_nodes(tree: &FileTree, left: usize, right: usize) -> std::cmp::Ordering {
    let left_node = &tree.nodes[left];
    let right_node = &tree.nodes[right];
    if left_node.kind != right_node.kind {
        return if left_node.kind == FileTreeKind::Directory {
            std::cmp::Ordering::Less
        } else {
            std::cmp::Ordering::Greater
        };
    }
    left_node.name.cmp(&right_node.name).then(left_node.id.cmp(&right_node.id))
}

pub fn flatten_file_tree(tree: &FileTree, expanded: &HashSet<usize>) -> Vec<FileTreeRow> {
    let mut rows = Vec::new();
    for &root in &tree.roots {
        visit_file_tree_node(tree, root, 0, expanded, &mut rows);
    }
    rows
}

fn visit_file_tree_node(
    tree: &FileTree,
    id: usize,
    depth: usize,
    expanded: &HashSet<usize>,
    rows: &mut Vec<FileTreeRow>,
) {
    let node = &tree.nodes[id];
    if node.kind == FileTreeKind::File {
        rows.push(FileTreeRow {
            id: node.id,
            depth,
            kind: node.kind.clone(),
            name: node.name.clone(),
            file_index: node.file_index,
        });
        return;
    }

    let chain = collapsed_directory_chain(tree, node.id);
    let last = chain.last().unwrap();
    rows.push(FileTreeRow {
        id: node.id,
        depth,
        kind: node.kind.clone(),
        name: chain.iter().map(|n| n.name.clone()).collect::<Vec<_>>().join("/"),
        file_index: node.file_index,
    });
    if expanded.contains(&node.id) {
        for &child in &last.children {
            visit_file_tree_node(tree, child, depth + 1, expanded, rows);
        }
    }
}

fn collapsed_directory_chain(tree: &FileTree, id: usize) -> Vec<&FileTreeNode> {
    let node = &tree.nodes[id];
    if node.children.len() == 1 {
        let child = &tree.nodes[node.children[0]];
        if child.kind == FileTreeKind::Directory {
            let mut chain = vec![node];
            chain.extend(collapsed_directory_chain(tree, child.id));
            return chain;
        }
    }
    vec![node]
}

pub fn all_expanded_directories(tree: &FileTree) -> HashSet<usize> {
    tree.nodes
        .iter()
        .filter(|n| n.kind == FileTreeKind::Directory)
        .map(|n| n.id)
        .collect()
}

pub fn toggle_directory(tree: &FileTree, expanded: &HashSet<usize>, selected: Option<usize>) -> HashSet<usize> {
    let selected = match selected {
        Some(s) => s,
        None => return expanded.clone(),
    };
    if tree.nodes.get(selected).map(|n| &n.kind) != Some(&FileTreeKind::Directory) {
        return expanded.clone();
    }
    let mut next = expanded.clone();
    if next.contains(&selected) { next.remove(&selected); } else { next.insert(selected); }
    next
}

pub fn set_directory_expanded(
    tree: &FileTree,
    expanded: &HashSet<usize>,
    selected: Option<usize>,
    value: bool,
) -> HashSet<usize> {
    let selected = match selected {
        Some(s) => s,
        None => return expanded.clone(),
    };
    if tree.nodes.get(selected).map(|n| &n.kind) != Some(&FileTreeKind::Directory) {
        return expanded.clone();
    }
    let mut next = expanded.clone();
    if value { next.insert(selected); } else { next.remove(&selected); }
    next
}

pub fn move_selection(rows: &[FileTreeRow], selected: Option<usize>, offset: i32) -> Option<usize> {
    if rows.is_empty() { return None }
    let index = match selected {
        None => return Some(rows[0].id),
        Some(id) => rows.iter().position(|r| r.id == id),
    };
    let index = match index {
        None => return Some(rows[0].id),
        Some(i) => i as i32,
    };
    let next = (index + offset).max(0).min(rows.len() as i32 - 1) as usize;
    Some(rows[next].id)
}

pub fn move_selection_to_first_child(rows: &[FileTreeRow], selected: Option<usize>) -> Option<usize> {
    let index = match selected {
        None => return selected,
        Some(id) => rows.iter().position(|r| r.id == id),
    };
    let index = match index {
        None => return selected,
        Some(i) => i,
    };
    let row = &rows[index];
    if row.kind != FileTreeKind::Directory { return selected }
    if let Some(child) = rows.get(index + 1) {
        if child.depth > row.depth { return Some(child.id); }
    }
    selected
}

pub fn move_selection_to_parent(rows: &[FileTreeRow], selected: Option<usize>) -> Option<usize> {
    let index = match selected {
        None => return selected,
        Some(id) => rows.iter().position(|r| r.id == id),
    };
    let index = match index {
        None => return selected,
        Some(i) => i,
    };
    let row = &rows[index];
    if row.depth == 0 { return selected }
    rows.iter()
        .enumerate()
        .filter(|(i, r)| *i < index && r.depth < row.depth)
        .last()
        .map(|(_, r)| r.id)
        .or(selected)
}

pub fn file_tree_file_selection(tree: &FileTree, file_index: usize) -> Option<(usize, HashSet<usize>)> {
    let node = tree.nodes.iter().find(|n| n.kind == FileTreeKind::File && n.file_index == Some(file_index))?;
    let mut expanded = HashSet::new();
    let mut parent = node.parent;
    while let Some(p) = parent {
        expanded.insert(p);
        parent = tree.nodes.get(p).and_then(|n| n.parent);
    }
    Some((node.id, expanded))
}

pub fn single_patch_file_index(
    selected: Option<usize>,
    active: Option<usize>,
    current: Option<usize>,
    first: Option<usize>,
) -> Option<usize> {
    selected.or(active).or(current).or(first)
}

pub fn ordered_patch_file_indexes(rows: &[FileTreeRow]) -> Vec<usize> {
    rows.iter().filter_map(|r| r.file_index).collect()
}

pub fn show_diff_viewer_file_tree(show_file_tree: bool, file_count: usize) -> bool {
    show_file_tree && file_count > 0
}

pub fn move_patch_file_index(file_indexes: &[usize], current: Option<usize>, offset: i32) -> Option<usize> {
    if file_indexes.is_empty() { return None }
    let index = match current {
        None => return Some(file_indexes[0]),
        Some(c) => file_indexes.iter().position(|&f| f == c),
    };
    let index = match index {
        None => return Some(file_indexes[0]),
        Some(i) => i as i32,
    };
    let next = (index + offset).max(0).min(file_indexes.len() as i32 - 1) as usize;
    Some(file_indexes[next])
}

pub fn has_later_sibling(rows: &[FileTreeRow], index: usize, depth: usize) -> bool {
    rows[index + 1..].iter().find(|r| r.depth <= depth).map(|r| r.depth == depth).unwrap_or(false)
}

pub fn file_tree_row_prefix(rows: &[FileTreeRow], index: usize, row: &FileTreeRow, expanded: &HashSet<usize>) -> String {
    let mut indentation = String::new();
    for depth in 0..row.depth {
        if depth == 0 && !has_later_sibling(rows, 0, 0) {
            indentation.push(' ');
        } else {
            indentation.push_str(if has_later_sibling(rows, index, depth) { "│  " } else { "   " });
        }
    }
    let top_root = index == 0 && row.depth == 0;
    let branch = if top_root {
        " "
    } else if has_later_sibling(rows, index, row.depth) {
        "├─ "
    } else {
        "└─ "
    };
    let marker = if row.kind == FileTreeKind::Directory {
        if !expanded.contains(&row.id) { "▸ " } else { "▾ " }
    } else {
        ""
    };
    format!("{}{}{}", indentation, branch, marker)
}

pub fn file_tree_row_status(row: &FileTreeRow, files: &[FileTreeItem], reviewed: bool) -> String {
    if row.file_index.is_none() { return String::new() }
    let file_index = row.file_index.unwrap();
    let status = files.get(file_index).and_then(|f| f.status.as_ref());
    let marker = match status {
        Some(DiffFileStatus::Modified) => 'M',
        Some(DiffFileStatus::Added) => 'A',
        Some(DiffFileStatus::Deleted) => 'D',
        _ => '?',
    };
    let check = if reviewed { '✓' } else { ' ' };
    format!("{}{}", check, marker)
}

#[derive(Clone, Copy)]
pub enum PanelBorder {
    Start,
    End,
    Both,
    None,
}

pub fn panel_border_sides(axis: &str, border: PanelBorder) -> Vec<&'static str> {
    if axis == "x" {
        match border {
            PanelBorder::Both => vec!["top", "bottom"],
            PanelBorder::Start => vec!["top"],
            PanelBorder::End => vec!["bottom"],
            PanelBorder::None => vec![],
        }
    } else {
        match border {
            PanelBorder::Both => vec!["left", "right"],
            PanelBorder::Start => vec!["left"],
            PanelBorder::End => vec!["right"],
            PanelBorder::None => vec![],
        }
    }
}

pub fn horizontal_edge(edge: &str, side: &str) -> &'static str {
    match edge {
        "edge" => if side == "start" { "├" } else { "┤" },
        "edge-in" => "┴",
        _ => "┬",
    }
}

pub fn vertical_edge(edge: &str, side: &str) -> &'static str {
    match edge {
        "edge" => if side == "start" { "┬" } else { "┴" },
        "edge-in" => "┤",
        _ => "├",
    }
}

pub const DIFF_COMMANDS: &[&str] = &[
    "diff.close", "diff.down", "diff.up", "diff.page.down", "diff.page.up",
    "diff.toggle", "diff.expand", "diff.expand_all", "diff.collapse",
    "diff.next_hunk", "diff.previous_hunk", "diff.next_file", "diff.previous_file",
    "diff.mark_reviewed", "diff.switch_focus", "diff.toggle_file_tree",
    "diff.single_patch", "diff.switch_source", "diff.toggle_view", "diff.help",
];

pub fn diff_help_rows() -> Vec<(&'static str, &'static str, &'static str)> {
    vec![
        ("q", "Close viewer", "Quit the diff viewer"),
        ("", "Focus file tree", "Move keyboard focus between the file tree and patch pane"),
        ("", "Next hunk", "Jump to the next diff hunk"),
        ("", "Previous hunk", "Jump to the previous diff hunk"),
        ("", "Next file", "Select the next changed file in file-tree order"),
        ("", "Previous file", "Select the previous changed file in file-tree order"),
        ("", "Toggle file tree", "Show or hide the file tree sidebar"),
        ("", "Toggle patches", "Switch between one selected patch and all patches"),
        ("", "Switch source", "Choose working tree, main branch, or last-turn changes"),
        ("", "Toggle view", "Switch between split and unified diff layout"),
        ("", "Expand all folders", "Open every folder in the file tree"),
        ("", "Mark reviewed", "Toggle reviewed state for the selected file"),
    ]
}
