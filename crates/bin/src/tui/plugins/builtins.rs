use crate::tui::plugins::diff_viewer::DiffViewerPlugin;
use crate::tui::plugins::home::{HomeFooterPlugin, HomeTipsPlugin};
use crate::tui::plugins::notifications::NotificationsPlugin;
use crate::tui::plugins::sidebar::{
    SidebarContextPlugin, SidebarFilesPlugin, SidebarFooterPlugin, SidebarLspPlugin, SidebarMcpPlugin,
    SidebarTodoPlugin,
};
use crate::tui::plugins::system::PluginManagerPlugin;
use crate::tui::plugins::which_key::WhichKeyPlugin;

pub const EXPERIMENTAL_EVENT_SYSTEM: bool = false;

pub struct BuiltinTuiPlugin {
    pub id: &'static str,
    pub enabled: bool,
    pub order: u32,
}

impl BuiltinTuiPlugin {
    pub const fn new(id: &'static str) -> Self {
        Self { id, enabled: true, order: 100 }
    }
    pub const fn disabled(id: &'static str) -> Self {
        Self { id, enabled: false, order: 100 }
    }
    pub const fn with_order(mut self, order: u32) -> Self {
        self.order = order;
        self
    }
}

pub fn create_builtin_plugins(experimental_event_system: bool) -> Vec<BuiltinTuiPlugin> {
    let _ = experimental_event_system;
    vec![
        HomeFooterPlugin::builtin(),
        HomeTipsPlugin::builtin(),
        SidebarContextPlugin::builtin(),
        SidebarMcpPlugin::builtin(),
        SidebarLspPlugin::builtin(),
        SidebarTodoPlugin::builtin(),
        SidebarFilesPlugin::builtin(),
        SidebarFooterPlugin::builtin(),
        NotificationsPlugin::builtin(),
        PluginManagerPlugin::builtin(),
        WhichKeyPlugin::builtin(),
        DiffViewerPlugin::builtin(),
    ]
}

pub fn plugin_ids() -> Vec<&'static str> {
    create_builtin_plugins(EXPERIMENTAL_EVENT_SYSTEM)
        .into_iter()
        .map(|p| p.id)
        .collect()
}
