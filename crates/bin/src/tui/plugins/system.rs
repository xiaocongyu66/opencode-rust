use crate::tui::plugins::builtins::BuiltinTuiPlugin;

pub const PLUGIN_MANAGER_ID: &str = "internal:plugin-manager";

pub struct PluginManagerPlugin;

impl PluginManagerPlugin {
    pub fn builtin() -> BuiltinTuiPlugin {
        BuiltinTuiPlugin::new(PLUGIN_MANAGER_ID).with_order(100)
    }

    pub fn id() -> &'static str { PLUGIN_MANAGER_ID }
}

#[derive(Clone, Debug)]
pub struct TuiPluginStatus {
    pub id: String,
    pub enabled: bool,
    pub active: bool,
    pub source: PluginSource,
    pub spec: String,
}

#[derive(Clone, Debug, PartialEq)]
pub enum PluginSource {
    Internal,
    External,
}

impl PluginSource {
    pub fn from_str(s: &str) -> Self {
        match s {
            "internal" => Self::Internal,
            _ => Self::External,
        }
    }
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Internal => "internal",
            Self::External => "external",
        }
    }
}

pub fn state_label(item: &TuiPluginStatus) -> &'static str {
    if !item.enabled { return "disabled" }
    if item.active { "active" } else { "inactive" }
}

pub fn state_color(item: &TuiPluginStatus) -> &'static str {
    if !item.enabled { return "textMuted" }
    if item.active { "success" } else { "error" }
}

pub fn source(spec: &str) -> Option<String> {
    if spec.starts_with("file://") {
        return Some(spec.strip_prefix("file://").unwrap_or(spec).to_string());
    }
    None
}

pub fn meta(item: &TuiPluginStatus, width: usize) -> String {
    if item.source == PluginSource::Internal {
        return if width >= 120 { "Built-in plugin".to_string() } else { "Built-in".to_string() };
    }
    source(&item.spec).unwrap_or_else(|| item.spec.clone())
}

pub fn sort_plugins(list: &mut [TuiPluginStatus]) {
    list.sort_by(|a, b| {
        let a_internal = (a.source == PluginSource::Internal) as u8;
        let b_internal = (b.source == PluginSource::Internal) as u8;
        b_internal
            .cmp(&a_internal)
            .then_with(|| a.id.cmp(&b.id))
    });
}

pub fn dialog_size(width: usize) -> &'static str {
    if width >= 128 { "xlarge" }
    else if width >= 96 { "large" }
    else { "medium" }
}

pub fn row_disabled(item: &TuiPluginStatus) -> bool {
    item.id == PLUGIN_MANAGER_ID
}

pub fn row_category(item: &TuiPluginStatus) -> &'static str {
    if item.source == PluginSource::Internal { "Internal" } else { "External" }
}

pub struct InstallOptions {
    pub global: bool,
    pub busy: bool,
    pub module: String,
}

impl InstallOptions {
    pub fn new() -> Self {
        Self { global: false, busy: false, module: String::new() }
    }
}

impl Default for InstallOptions {
    fn default() -> Self { Self::new() }
}
