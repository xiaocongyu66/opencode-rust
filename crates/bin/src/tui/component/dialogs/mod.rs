//! Dialog components — ported from tui/src/component/dialog-*.tsx
//!
//! Each dialog is a struct with `render` + `handle_key` methods,
//! built on top of `crate::tui::component::dialog::Dialog`.

pub mod session_list;
pub mod session_rename;
pub mod model;
pub mod agent;
pub mod theme_list;
pub mod provider;
pub mod skill;
pub mod debug;
pub mod mcp;
pub mod move_session;
pub mod retry_action;
pub mod session_delete_failed;
pub mod stash;
pub mod status;
pub mod tag;
pub mod variant;
pub mod workspace_create;
pub mod workspace_file_changes;
pub mod workspace_list;
pub mod workspace_unavailable;
pub mod console_org;

// Re-export the most commonly used types
pub use session_list::DialogSessionList;
pub use session_rename::DialogSessionRename;
pub use model::DialogModel;
pub use agent::DialogAgent;
pub use theme_list::DialogThemeList;
pub use provider::DialogProvider;
pub use skill::DialogSkill;
