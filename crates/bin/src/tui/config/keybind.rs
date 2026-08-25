use std::collections::HashMap;

pub const LEADER_DEFAULT: &str = "ctrl+x";

pub type KeybindName = &'static str;

#[derive(Debug, Clone, PartialEq)]
pub enum BindingValue {
    Disabled,
    None,
    Single(String),
    Items(Vec<BindingValue>),
}

impl BindingValue {
    pub fn as_keys(&self) -> Vec<String> {
        match self {
            BindingValue::Disabled | BindingValue::None => Vec::new(),
            BindingValue::Single(s) => s.split(',').map(|p| p.trim().to_string()).collect(),
            BindingValue::Items(items) => {
                let mut out = Vec::new();
                for item in items {
                    out.extend(item.as_keys());
                }
                out
            }
        }
    }

    pub fn is_disabled(&self) -> bool {
        matches!(self, BindingValue::Disabled | BindingValue::None)
    }
}

#[derive(Debug, Clone)]
pub struct Keybind {
    pub name: &'static str,
    pub command: &'static str,
    pub default: BindingValue,
    pub description: &'static str,
    pub category: KeybindCategory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeybindCategory {
    App,
    Command,
    Help,
    Diff,
    Editor,
    Theme,
    Sidebar,
    Session,
    Stash,
    Model,
    Mcp,
    Provider,
    Console,
    Agent,
    Variant,
    Messages,
    Prompt,
    Input,
    History,
    Dialog,
    Autocomplete,
    Permission,
    Plugins,
    Terminal,
    Tips,
    WhichKey,
}

struct Definition {
    default: &'static str,
    description: &'static str,
}

const fn def(default: &'static str, description: &'static str) -> Definition {
    Definition { default, description }
}

pub static DEFINITIONS: &[(&'static str, &'static str, &'static str, KeybindCategory)] = &[
    ("leader", "ctrl+x", "Leader key for keybind combinations", KeybindCategory::App),
    ("app_exit", "ctrl+c,ctrl+d,<leader>q", "Exit the application", KeybindCategory::App),
    ("app_debug", "none", "Toggle debug panel", KeybindCategory::App),
    ("app_console", "none", "Toggle console", KeybindCategory::App),
    ("app_heap_snapshot", "none", "Write heap snapshot", KeybindCategory::App),
    ("app_toggle_animations", "none", "Toggle animations", KeybindCategory::App),
    ("app_toggle_file_context", "none", "Toggle file context", KeybindCategory::App),
    ("app_toggle_diffwrap", "none", "Toggle diff wrapping", KeybindCategory::App),
    ("app_toggle_paste_summary", "none", "Toggle paste summary", KeybindCategory::App),
    ("app_toggle_session_directory_filter", "none", "Toggle session directory filtering", KeybindCategory::App),
    ("command_list", "ctrl+p", "List available commands", KeybindCategory::Command),
    ("help_show", "none", "Open help dialog", KeybindCategory::Help),
    ("docs_open", "none", "Open documentation", KeybindCategory::Help),
    ("diff_open", "none", "Open diff viewer", KeybindCategory::Diff),
    ("diff_close", "escape,q", "Close diff viewer", KeybindCategory::Diff),
    ("diff_toggle", "enter,space", "Toggle diff viewer item", KeybindCategory::Diff),
    ("diff_expand", "right", "Expand diff viewer item", KeybindCategory::Diff),
    ("diff_expand_all", "E", "Expand all diff viewer folders", KeybindCategory::Diff),
    ("diff_collapse", "left", "Collapse diff viewer item", KeybindCategory::Diff),
    ("diff_switch_focus", "tab", "Switch diff viewer focus", KeybindCategory::Diff),
    ("diff_next_hunk", "]", "Jump to next diff hunk", KeybindCategory::Diff),
    ("diff_previous_hunk", "[", "Jump to previous diff hunk", KeybindCategory::Diff),
    ("diff_next_file", "n", "Jump to next diff file", KeybindCategory::Diff),
    ("diff_previous_file", "p", "Jump to previous diff file", KeybindCategory::Diff),
    ("diff_toggle_file_tree", "b", "Toggle diff viewer file tree", KeybindCategory::Diff),
    ("diff_single_patch", "s", "Toggle single patch view", KeybindCategory::Diff),
    ("diff_switch_source", "d", "Switch diff viewer source", KeybindCategory::Diff),
    ("diff_toggle_view", "v", "Toggle diff viewer split or unified view", KeybindCategory::Diff),
    ("diff_help", "?", "Show more diff viewer shortcuts", KeybindCategory::Diff),
    ("editor_open", "<leader>e", "Open external editor", KeybindCategory::Editor),
    ("theme_list", "<leader>t", "List available themes", KeybindCategory::Theme),
    ("theme_switch_mode", "none", "Switch between light and dark theme mode", KeybindCategory::Theme),
    ("theme_mode_lock", "none", "Lock or unlock theme mode", KeybindCategory::Theme),
    ("sidebar_toggle", "<leader>b", "Toggle sidebar", KeybindCategory::Sidebar),
    ("scrollbar_toggle", "none", "Toggle session scrollbar", KeybindCategory::Sidebar),
    ("status_view", "<leader>s", "View status", KeybindCategory::Sidebar),
    ("debug_view", "none", "View debug info", KeybindCategory::Sidebar),
    ("session_export", "<leader>x", "Export session to editor", KeybindCategory::Session),
    ("session_copy", "none", "Copy session transcript", KeybindCategory::Session),
    ("session_move", "none", "Move session", KeybindCategory::Session),
    ("session_new", "<leader>n", "Create a new session", KeybindCategory::Session),
    ("session_list", "<leader>l", "List all sessions", KeybindCategory::Session),
    ("session_timeline", "<leader>g", "Show session timeline", KeybindCategory::Session),
    ("session_fork", "none", "Fork session from message", KeybindCategory::Session),
    ("session_rename", "ctrl+r", "Rename session", KeybindCategory::Session),
    ("session_delete", "ctrl+d", "Delete session", KeybindCategory::Session),
    ("session_share", "none", "Share current session", KeybindCategory::Session),
    ("session_unshare", "none", "Unshare current session", KeybindCategory::Session),
    ("session_interrupt", "escape", "Interrupt current session", KeybindCategory::Session),
    ("session_background", "ctrl+b", "Background synchronous subagents", KeybindCategory::Session),
    ("session_compact", "<leader>c", "Compact the session", KeybindCategory::Session),
    ("session_toggle_timestamps", "none", "Toggle message timestamps", KeybindCategory::Session),
    ("session_toggle_generic_tool_output", "none", "Toggle generic tool output", KeybindCategory::Session),
    ("session_queued_prompts", "<leader>q", "Manage queued prompts", KeybindCategory::Session),
    ("session_child_first", "<leader>down", "Go to first child session", KeybindCategory::Session),
    ("session_child_cycle", "right", "Go to next child session", KeybindCategory::Session),
    ("session_child_cycle_reverse", "left", "Go to previous child session", KeybindCategory::Session),
    ("session_parent", "up", "Go to parent session", KeybindCategory::Session),
    ("session_pin_toggle", "ctrl+f", "Pin or unpin session in the session list", KeybindCategory::Session),
    ("session_quick_switch_1", "<leader>1", "Switch to session in quick slot 1", KeybindCategory::Session),
    ("session_quick_switch_2", "<leader>2", "Switch to session in quick slot 2", KeybindCategory::Session),
    ("session_quick_switch_3", "<leader>3", "Switch to session in quick slot 3", KeybindCategory::Session),
    ("session_quick_switch_4", "<leader>4", "Switch to session in quick slot 4", KeybindCategory::Session),
    ("session_quick_switch_5", "<leader>5", "Switch to session in quick slot 5", KeybindCategory::Session),
    ("session_quick_switch_6", "<leader>6", "Switch to session in quick slot 6", KeybindCategory::Session),
    ("session_quick_switch_7", "<leader>7", "Switch to session in quick slot 7", KeybindCategory::Session),
    ("session_quick_switch_8", "<leader>8", "Switch to session in quick slot 8", KeybindCategory::Session),
    ("session_quick_switch_9", "<leader>9", "Switch to session in quick slot 9", KeybindCategory::Session),
    ("stash_delete", "ctrl+d", "Delete stash entry", KeybindCategory::Stash),
    ("model_provider_list", "ctrl+a", "Open provider list from model dialog", KeybindCategory::Model),
    ("model_favorite_toggle", "ctrl+f", "Toggle model favorite status", KeybindCategory::Model),
    ("model_list", "<leader>m", "List available models", KeybindCategory::Model),
    ("model_cycle_recent", "f2", "Next recently used model", KeybindCategory::Model),
    ("model_cycle_recent_reverse", "shift+f2", "Previous recently used model", KeybindCategory::Model),
    ("model_cycle_favorite", "none", "Next favorite model", KeybindCategory::Model),
    ("model_cycle_favorite_reverse", "none", "Previous favorite model", KeybindCategory::Model),
    ("mcp_list", "none", "List MCP servers", KeybindCategory::Mcp),
    ("provider_connect", "none", "Connect provider", KeybindCategory::Provider),
    ("console_org_switch", "none", "Switch console organization", KeybindCategory::Console),
    ("agent_list", "<leader>a", "List agents", KeybindCategory::Agent),
    ("agent_cycle", "tab", "Next agent", KeybindCategory::Agent),
    ("agent_cycle_reverse", "shift+tab", "Previous agent", KeybindCategory::Agent),
    ("variant_cycle", "ctrl+t", "Cycle model variants", KeybindCategory::Variant),
    ("variant_list", "none", "List model variants", KeybindCategory::Variant),
    ("messages_page_up", "pageup,ctrl+alt+b", "Scroll messages up by one page", KeybindCategory::Messages),
    ("messages_page_down", "pagedown,ctrl+alt+f", "Scroll messages down by one page", KeybindCategory::Messages),
    ("messages_line_up", "ctrl+alt+y", "Scroll messages up by one line", KeybindCategory::Messages),
    ("messages_line_down", "ctrl+alt+e", "Scroll messages down by one line", KeybindCategory::Messages),
    ("messages_half_page_up", "ctrl+alt+u", "Scroll messages up by half page", KeybindCategory::Messages),
    ("messages_half_page_down", "ctrl+alt+d", "Scroll messages down by half page", KeybindCategory::Messages),
    ("messages_first", "ctrl+g,home", "Navigate to first message", KeybindCategory::Messages),
    ("messages_last", "ctrl+alt+g,end", "Navigate to last message", KeybindCategory::Messages),
    ("messages_next", "none", "Navigate to next message", KeybindCategory::Messages),
    ("messages_previous", "none", "Navigate to previous message", KeybindCategory::Messages),
    ("messages_last_user", "none", "Navigate to last user message", KeybindCategory::Messages),
    ("messages_copy", "<leader>y", "Copy message", KeybindCategory::Messages),
    ("messages_undo", "<leader>u", "Undo message", KeybindCategory::Messages),
    ("messages_redo", "<leader>r", "Redo message", KeybindCategory::Messages),
    ("messages_toggle_conceal", "<leader>h", "Toggle code block concealment in messages", KeybindCategory::Messages),
    ("tool_details", "none", "Toggle tool details visibility", KeybindCategory::Messages),
    ("display_thinking", "none", "Toggle thinking blocks visibility", KeybindCategory::Messages),
    ("prompt_submit", "none", "Submit prompt", KeybindCategory::Prompt),
    ("prompt_editor_context_clear", "none", "Clear editor context", KeybindCategory::Prompt),
    ("prompt_skills", "none", "Open skill selector", KeybindCategory::Prompt),
    ("prompt_stash", "none", "Stash prompt", KeybindCategory::Prompt),
    ("prompt_stash_pop", "none", "Pop stashed prompt", KeybindCategory::Prompt),
    ("prompt_stash_list", "none", "List stashed prompts", KeybindCategory::Prompt),
    ("workspace_set", "none", "Set workspace", KeybindCategory::Prompt),
    ("input_clear", "ctrl+c", "Clear input field", KeybindCategory::Input),
    ("input_paste", "ctrl+v", "Paste from clipboard", KeybindCategory::Input),
    ("input_submit", "return", "Submit input", KeybindCategory::Input),
    ("input_newline", "shift+return,ctrl+return,alt+return,ctrl+j", "Insert newline in input", KeybindCategory::Input),
    ("input_move_left", "left,ctrl+b", "Move cursor left in input", KeybindCategory::Input),
    ("input_move_right", "right,ctrl+f", "Move cursor right in input", KeybindCategory::Input),
    ("input_move_up", "up", "Move cursor up in input", KeybindCategory::Input),
    ("input_move_down", "down", "Move cursor down in input", KeybindCategory::Input),
    ("input_select_left", "shift+left", "Select left in input", KeybindCategory::Input),
    ("input_select_right", "shift+right", "Select right in input", KeybindCategory::Input),
    ("input_select_up", "shift+up", "Select up in input", KeybindCategory::Input),
    ("input_select_down", "shift+down", "Select down in input", KeybindCategory::Input),
    ("input_line_home", "ctrl+a", "Move to start of line in input", KeybindCategory::Input),
    ("input_line_end", "ctrl+e", "Move to end of line in input", KeybindCategory::Input),
    ("input_select_line_home", "ctrl+shift+a", "Select to start of line in input", KeybindCategory::Input),
    ("input_select_line_end", "ctrl+shift+e", "Select to end of line in input", KeybindCategory::Input),
    ("input_visual_line_home", "alt+a", "Move to start of visual line in input", KeybindCategory::Input),
    ("input_visual_line_end", "alt+e", "Move to end of visual line in input", KeybindCategory::Input),
    ("input_select_visual_line_home", "alt+shift+a", "Select to start of visual line in input", KeybindCategory::Input),
    ("input_select_visual_line_end", "alt+shift+e", "Select to end of visual line in input", KeybindCategory::Input),
    ("input_buffer_home", "home", "Move to start of buffer in input", KeybindCategory::Input),
    ("input_buffer_end", "end", "Move to end of buffer in input", KeybindCategory::Input),
    ("input_select_buffer_home", "shift+home", "Select to start of buffer in input", KeybindCategory::Input),
    ("input_select_buffer_end", "shift+end", "Select to end of buffer in input", KeybindCategory::Input),
    ("input_delete_line", "ctrl+shift+d", "Delete line in input", KeybindCategory::Input),
    ("input_delete_to_line_end", "ctrl+k", "Delete to end of line in input", KeybindCategory::Input),
    ("input_delete_to_line_start", "ctrl+u", "Delete to start of line in input", KeybindCategory::Input),
    ("input_backspace", "backspace,shift+backspace", "Backspace in input", KeybindCategory::Input),
    ("input_delete", "ctrl+d,delete,shift+delete", "Delete character in input", KeybindCategory::Input),
    ("input_undo", "ctrl+-,super+z", "Undo in input", KeybindCategory::Input),
    ("input_redo", "ctrl+.,super+shift+z", "Redo in input", KeybindCategory::Input),
    ("input_word_forward", "alt+f,alt+right,ctrl+right", "Move word forward in input", KeybindCategory::Input),
    ("input_word_backward", "alt+b,alt+left,ctrl+left", "Move word backward in input", KeybindCategory::Input),
    ("input_select_word_forward", "alt+shift+f,alt+shift+right", "Select word forward in input", KeybindCategory::Input),
    ("input_select_word_backward", "alt+shift+b,alt+shift+left", "Select word backward in input", KeybindCategory::Input),
    ("input_delete_word_forward", "alt+d,alt+delete,ctrl+delete", "Delete word forward in input", KeybindCategory::Input),
    ("input_delete_word_backward", "ctrl+w,ctrl+backspace,alt+backspace", "Delete word backward in input", KeybindCategory::Input),
    ("input_select_all", "super+a", "Select all in input", KeybindCategory::Input),
    ("history_previous", "up", "Previous history item", KeybindCategory::History),
    ("history_next", "down", "Next history item", KeybindCategory::History),
    ("dialog.select.prev", "up,ctrl+p", "Move to previous dialog item", KeybindCategory::Dialog),
    ("dialog.select.next", "down,ctrl+n", "Move to next dialog item", KeybindCategory::Dialog),
    ("dialog.select.page_up", "pageup", "Move up one page in dialog", KeybindCategory::Dialog),
    ("dialog.select.page_down", "pagedown", "Move down one page in dialog", KeybindCategory::Dialog),
    ("dialog.select.home", "home", "Move to first dialog item", KeybindCategory::Dialog),
    ("dialog.select.end", "end", "Move to last dialog item", KeybindCategory::Dialog),
    ("dialog.select.submit", "return", "Submit selected dialog item", KeybindCategory::Dialog),
    ("dialog.prompt.submit", "return", "Submit dialog prompt", KeybindCategory::Dialog),
    ("dialog.mcp.toggle", "space", "Toggle MCP in MCP dialog", KeybindCategory::Dialog),
    ("dialog.move_session.new", "ctrl+m", "New project copy", KeybindCategory::Dialog),
    ("dialog.move_session.delete", "ctrl+d", "Delete project copy", KeybindCategory::Dialog),
    ("dialog.move_session.refresh", "ctrl+r", "Refresh project copies", KeybindCategory::Dialog),
    ("prompt.autocomplete.prev", "up,ctrl+p", "Move to previous autocomplete item", KeybindCategory::Autocomplete),
    ("prompt.autocomplete.next", "down,ctrl+n", "Move to next autocomplete item", KeybindCategory::Autocomplete),
    ("prompt.autocomplete.hide", "escape", "Hide autocomplete", KeybindCategory::Autocomplete),
    ("prompt.autocomplete.select", "return", "Select autocomplete item", KeybindCategory::Autocomplete),
    ("prompt.autocomplete.complete", "tab", "Complete autocomplete item", KeybindCategory::Autocomplete),
    ("permission.prompt.fullscreen", "ctrl+f", "Toggle permission prompt fullscreen", KeybindCategory::Permission),
    ("plugins.toggle", "space", "Toggle plugin", KeybindCategory::Plugins),
    ("dialog.plugins.install", "shift+i", "Install plugin from plugin dialog", KeybindCategory::Plugins),
    ("terminal_suspend", "ctrl+z", "Suspend terminal", KeybindCategory::Terminal),
    ("terminal_title_toggle", "none", "Toggle terminal title", KeybindCategory::Terminal),
    ("tips_toggle", "<leader>h", "Toggle tips on home screen", KeybindCategory::Tips),
    ("plugin_manager", "none", "Open plugin manager dialog", KeybindCategory::Plugins),
    ("plugin_install", "none", "Install plugin", KeybindCategory::Plugins),
    ("which_key_toggle", "ctrl+alt+k", "Toggle which-key panel", KeybindCategory::WhichKey),
    ("which_key_layout_toggle", "ctrl+alt+shift+k", "Switch which-key layout", KeybindCategory::WhichKey),
    ("which_key_pending_toggle", "ctrl+alt+shift+p", "Toggle which-key pending preview", KeybindCategory::WhichKey),
    ("which_key_group_previous", "ctrl+alt+left,ctrl+alt+[", "Previous which-key group", KeybindCategory::WhichKey),
    ("which_key_group_next", "ctrl+alt+right,ctrl+alt+]", "Next which-key group", KeybindCategory::WhichKey),
    ("which_key_scroll_up", "ctrl+alt+up,ctrl+alt+p", "Scroll which-key up", KeybindCategory::WhichKey),
    ("which_key_scroll_down", "ctrl+alt+down,ctrl+alt+n", "Scroll which-key down", KeybindCategory::WhichKey),
    ("which_key_page_up", "ctrl+alt+pageup", "Page which-key up", KeybindCategory::WhichKey),
    ("which_key_page_down", "ctrl+alt+pagedown", "Page which-key down", KeybindCategory::WhichKey),
    ("which_key_home", "ctrl+alt+home", "Jump to first which-key binding", KeybindCategory::WhichKey),
    ("which_key_end", "ctrl+alt+end", "Jump to last which-key binding", KeybindCategory::WhichKey),
];

pub static COMMAND_MAP: &[(&'static str, &'static str)] = &[
    ("app_exit", "app.exit"),
    ("app_debug", "app.debug"),
    ("app_console", "app.console"),
    ("app_heap_snapshot", "app.heap_snapshot"),
    ("app_toggle_animations", "app.toggle.animations"),
    ("app_toggle_file_context", "app.toggle.file_context"),
    ("app_toggle_diffwrap", "app.toggle.diffwrap"),
    ("app_toggle_paste_summary", "app.toggle.paste_summary"),
    ("app_toggle_session_directory_filter", "app.toggle.session_directory_filter"),
    ("command_list", "command.palette.show"),
    ("help_show", "help.show"),
    ("docs_open", "docs.open"),
    ("diff_open", "diff.open"),
    ("diff_close", "diff.close"),
    ("diff_toggle", "diff.toggle"),
    ("diff_expand", "diff.expand"),
    ("diff_expand_all", "diff.expand_all"),
    ("diff_collapse", "diff.collapse"),
    ("diff_switch_focus", "diff.switch_focus"),
    ("diff_next_hunk", "diff.next_hunk"),
    ("diff_previous_hunk", "diff.previous_hunk"),
    ("diff_next_file", "diff.next_file"),
    ("diff_previous_file", "diff.previous_file"),
    ("diff_toggle_file_tree", "diff.toggle_file_tree"),
    ("diff_single_patch", "diff.single_patch"),
    ("diff_switch_source", "diff.switch_source"),
    ("diff_toggle_view", "diff.toggle_view"),
    ("diff_help", "diff.help"),
    ("editor_open", "prompt.editor"),
    ("theme_list", "theme.switch"),
    ("theme_switch_mode", "theme.switch_mode"),
    ("theme_mode_lock", "theme.mode.lock"),
    ("sidebar_toggle", "session.sidebar.toggle"),
    ("scrollbar_toggle", "session.toggle.scrollbar"),
    ("status_view", "opencode.status"),
    ("debug_view", "opencode.debug"),
    ("session_export", "session.export"),
    ("session_copy", "session.copy"),
    ("session_move", "session.move"),
    ("session_new", "session.new"),
    ("session_list", "session.list"),
    ("session_timeline", "session.timeline"),
    ("session_fork", "session.fork"),
    ("session_rename", "session.rename"),
    ("session_delete", "session.delete"),
    ("session_share", "session.share"),
    ("session_unshare", "session.unshare"),
    ("session_interrupt", "session.interrupt"),
    ("session_background", "session.background"),
    ("session_compact", "session.compact"),
    ("session_toggle_timestamps", "session.toggle.timestamps"),
    ("session_toggle_generic_tool_output", "session.toggle.generic_tool_output"),
    ("session_queued_prompts", "session.queued_prompts"),
    ("session_child_first", "session.child.first"),
    ("session_child_cycle", "session.child.next"),
    ("session_child_cycle_reverse", "session.child.previous"),
    ("session_parent", "session.parent"),
    ("session_pin_toggle", "session.pin.toggle"),
    ("session_quick_switch_1", "session.quick_switch.1"),
    ("session_quick_switch_2", "session.quick_switch.2"),
    ("session_quick_switch_3", "session.quick_switch.3"),
    ("session_quick_switch_4", "session.quick_switch.4"),
    ("session_quick_switch_5", "session.quick_switch.5"),
    ("session_quick_switch_6", "session.quick_switch.6"),
    ("session_quick_switch_7", "session.quick_switch.7"),
    ("session_quick_switch_8", "session.quick_switch.8"),
    ("session_quick_switch_9", "session.quick_switch.9"),
    ("stash_delete", "stash.delete"),
    ("model_provider_list", "model.dialog.provider"),
    ("model_favorite_toggle", "model.dialog.favorite"),
    ("model_list", "model.list"),
    ("model_cycle_recent", "model.cycle_recent"),
    ("model_cycle_recent_reverse", "model.cycle_recent_reverse"),
    ("model_cycle_favorite", "model.cycle_favorite"),
    ("model_cycle_favorite_reverse", "model.cycle_favorite_reverse"),
    ("mcp_list", "mcp.list"),
    ("provider_connect", "provider.connect"),
    ("console_org_switch", "console.org.switch"),
    ("agent_list", "agent.list"),
    ("agent_cycle", "agent.cycle"),
    ("agent_cycle_reverse", "agent.cycle.reverse"),
    ("variant_cycle", "variant.cycle"),
    ("variant_list", "variant.list"),
    ("messages_page_up", "session.page.up"),
    ("messages_page_down", "session.page.down"),
    ("messages_line_up", "session.line.up"),
    ("messages_line_down", "session.line.down"),
    ("messages_half_page_up", "session.half.page.up"),
    ("messages_half_page_down", "session.half.page.down"),
    ("messages_first", "session.first"),
    ("messages_last", "session.last"),
    ("messages_next", "session.message.next"),
    ("messages_previous", "session.message.previous"),
    ("messages_last_user", "session.messages_last_user"),
    ("messages_copy", "messages.copy"),
    ("messages_undo", "session.undo"),
    ("messages_redo", "session.redo"),
    ("messages_toggle_conceal", "session.toggle.conceal"),
    ("tool_details", "session.toggle.actions"),
    ("display_thinking", "session.toggle.thinking"),
    ("prompt_submit", "prompt.submit"),
    ("prompt_editor_context_clear", "prompt.editor_context.clear"),
    ("prompt_skills", "prompt.skills"),
    ("prompt_stash", "prompt.stash"),
    ("prompt_stash_pop", "prompt.stash.pop"),
    ("prompt_stash_list", "prompt.stash.list"),
    ("workspace_set", "workspace.set"),
    ("input_clear", "prompt.clear"),
    ("input_paste", "prompt.paste"),
    ("input_submit", "input.submit"),
    ("input_newline", "input.newline"),
    ("input_move_left", "input.move.left"),
    ("input_move_right", "input.move.right"),
    ("input_move_up", "input.move.up"),
    ("input_move_down", "input.move.down"),
    ("input_select_left", "input.select.left"),
    ("input_select_right", "input.select.right"),
    ("input_select_up", "input.select.up"),
    ("input_select_down", "input.select.down"),
    ("input_line_home", "input.line.home"),
    ("input_line_end", "input.line.end"),
    ("input_select_line_home", "input.select.line.home"),
    ("input_select_line_end", "input.select.line.end"),
    ("input_visual_line_home", "input.visual.line.home"),
    ("input_visual_line_end", "input.visual.line.end"),
    ("input_select_visual_line_home", "input.select.visual.line.home"),
    ("input_select_visual_line_end", "input.select.visual.line.end"),
    ("input_buffer_home", "input.buffer.home"),
    ("input_buffer_end", "input.buffer.end"),
    ("input_select_buffer_home", "input.select.buffer.home"),
    ("input_select_buffer_end", "input.select.buffer.end"),
    ("input_delete_line", "input.delete.line"),
    ("input_delete_to_line_end", "input.delete.to.line.end"),
    ("input_delete_to_line_start", "input.delete.to.line.start"),
    ("input_backspace", "input.backspace"),
    ("input_delete", "input.delete"),
    ("input_undo", "input.undo"),
    ("input_redo", "input.redo"),
    ("input_word_forward", "input.word.forward"),
    ("input_word_backward", "input.word.backward"),
    ("input_select_word_forward", "input.select.word.forward"),
    ("input_select_word_backward", "input.select.word.backward"),
    ("input_delete_word_forward", "input.delete.word.forward"),
    ("input_delete_word_backward", "input.delete.word.backward"),
    ("input_select_all", "input.select.all"),
    ("history_previous", "prompt.history.previous"),
    ("history_next", "prompt.history.next"),
    ("terminal_suspend", "terminal.suspend"),
    ("terminal_title_toggle", "terminal.title.toggle"),
    ("tips_toggle", "tips.toggle"),
    ("plugin_manager", "plugins.list"),
    ("plugin_install", "plugins.install"),
    ("which_key_toggle", "which-key.toggle"),
    ("which_key_layout_toggle", "which-key.layout.toggle"),
    ("which_key_pending_toggle", "which-key.pending.toggle"),
    ("which_key_group_previous", "which-key.group.previous"),
    ("which_key_group_next", "which-key.group.next"),
    ("which_key_scroll_up", "which-key.scroll.up"),
    ("which_key_scroll_down", "which-key.scroll.down"),
    ("which_key_page_up", "which-key.page.up"),
    ("which_key_page_down", "which-key.page.down"),
    ("which_key_home", "which-key.home"),
    ("which_key_end", "which-key.end"),
];

pub fn command_for(name: &str) -> Option<&'static str> {
    COMMAND_MAP
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, cmd)| *cmd)
}

pub fn name_for_command(command: &str) -> Option<&'static str> {
    COMMAND_MAP
        .iter()
        .find(|(_, cmd)| *cmd == command)
        .map(|(name, _)| *name)
}

pub fn description_for(name: &str) -> Option<&'static str> {
    DEFINITIONS.iter().find(|(n, _, _, _)| *n == name).map(|(_, _, desc, _)| *desc)
}

pub fn command_description(command: &str) -> Option<&'static str> {
    let name = name_for_command(command)?;
    description_for(name)
}

pub static COMMAND_DESCRIPTIONS: std::sync::LazyLock<HashMap<&'static str, &'static str>> =
    std::sync::LazyLock::new(|| {
        let mut map = HashMap::new();
        for (name, _, desc, _) in DEFINITIONS {
            let cmd = command_for(name).unwrap_or(*name);
            map.insert(cmd, *desc);
        }
        map
    });

pub fn default_value(name: &str) -> BindingValue {
    match DEFINITIONS.iter().find(|(n, _, _, _)| *n == name) {
        Some((_, default, _, _)) => parse_binding_string(default),
        None => BindingValue::None,
    }
}

fn parse_binding_string(s: &str) -> BindingValue {
    if s == "none" || s == "false" {
        return BindingValue::None;
    }
    BindingValue::Single(s.to_string())
}

pub fn parse(overrides: &HashMap<String, BindingValue>) -> Vec<Keybind> {
    let mut invalid: Vec<&String> = Vec::new();
    let valid_names: Vec<&'static str> = DEFINITIONS.iter().map(|(n, _, _, _)| *n).collect();

    for key in overrides.keys() {
        if !valid_names.contains(&key.as_str()) {
            invalid.push(key);
        }
    }
    if !invalid.is_empty() {
        let suffix = if invalid.len() == 1 { "" } else { "s" };
        let joined = invalid.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", ");
        panic!("Unrecognized keybind{}: {}", suffix, joined);
    }

    DEFINITIONS
        .iter()
        .map(|(name, default, desc, category)| {
            let binding = overrides
                .get(*name)
                .cloned()
                .unwrap_or_else(|| parse_binding_string(default));
            let command = command_for(name).unwrap_or(*name);
            Keybind {
                name,
                command,
                default: binding,
                description: *desc,
                category: *category,
            }
        })
        .collect()
}

pub fn unknown_keys(input: &HashMap<String, BindingValue>) -> Vec<String> {
    let valid: Vec<&'static str> = DEFINITIONS.iter().map(|(n, _, _, _)| *n).collect();
    input
        .keys()
        .filter(|k| !valid.contains(&k.as_str()))
        .cloned()
        .collect()
}

pub fn all_default_keybinds() -> Vec<Keybind> {
    parse(&HashMap::new())
}
