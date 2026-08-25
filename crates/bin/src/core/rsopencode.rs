//! `.rsopencode` configuration directory support.
//!
//! Provides a Claude/Codex-style local configuration directory:
//! - Global config: `~/.rsopencode/config.toml`
//! - Project config: `.rsopencode/project.toml`
//! - Session files: `.rsopencode/sessions/`
//!
//! The directory layout mirrors `.claude` / `.codex` conventions so users
//! can version-control project settings and keep per-user defaults separate.

use std::fs;
use std::io;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Directory name for the project-local configuration directory.
pub const DIR_NAME: &str = ".rsopencode";

/// Default global config file name.
pub const GLOBAL_CONFIG_FILE: &str = "config.toml";

/// Default project config file name.
pub const PROJECT_CONFIG_FILE: &str = "project.toml";

/// Session files subdirectory name.
pub const SESSIONS_DIR: &str = "sessions";

/// Default content written to `~/.rsopencode/config.toml` on first init.
///
/// All values are commented out so the file serves as documentation without
/// overriding `GlobalConfig::default()`. Users uncomment lines to activate them.
const DEFAULT_GLOBAL_CONFIG_CONTENT: &str = "\
# rsopencode 配置文件
# 基于 opencode 配置格式
#
# 取消注释并修改以下配置项以自定义你的 rsopencode 体验。

# 默认模型标识符 (例如 \"anthropic/claude-sonnet-4-5\")
# model = \"anthropic/claude-sonnet-4-5\"

# 默认 agent ID (例如 \"build\", \"plan\")
# defaultAgent = \"build\"

# Shell 命令 (默认使用系统 shell)
# shell = \"zsh\"

# 用户名 (显示在会话消息中)
# username = \"user\"

# 是否启用文件快照
# snapshots = true

# 语言设置 (\"en\" 或 \"zh\")
# locale = \"en\"

# 主题名称
# theme = \"dark\"
";

/// Default content written to `.rsopencode/project.toml` on first init.
const DEFAULT_PROJECT_CONFIG_CONTENT: &str = "\
# rsopencode 项目配置文件
# 放在 .rsopencode/project.toml，可提交到版本控制共享给团队
#
# 取消注释并修改以下配置项以覆盖全局配置。

# 项目模型覆盖
# model = \"anthropic/claude-sonnet-4-5\"

# 项目 agent 覆盖
# defaultAgent = \"build\"

# 项目 shell 覆盖
# shell = \"zsh\"

# 指令文件路径 (相对于项目根目录)
# instructions = [\"AGENTS.md\"]

# 启用的技能
# skills = [\"code-review\"]

# 是否启用文件快照
# snapshots = true
";

// ---------------------------------------------------------------------------
// Global configuration — `~/.rsopencode/config.toml`
// ---------------------------------------------------------------------------

/// Global user configuration stored at `~/.rsopencode/config.toml`.
///
/// Holds user-level defaults that apply to every project unless overridden
/// by a project-level `.rsopencode/project.toml`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GlobalConfig {
    /// Default model identifier (e.g. `"anthropic/claude-sonnet-4-5"`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// Default agent ID to use when creating sessions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_agent: Option<String>,
    /// Shell to use for bash tool execution.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shell: Option<String>,
    /// Username displayed in session messages.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    /// Whether to enable file snapshots.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snapshots: Option<bool>,
    /// Preferred locale (`"en"` or `"zh"`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locale: Option<String>,
    /// Theme name for the TUI.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub theme: Option<String>,
}

impl GlobalConfig {
    /// Parses a TOML string into a [`GlobalConfig`].
    pub fn from_toml_str(s: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(s)
    }

    /// Serializes to a TOML string.
    pub fn to_toml_string(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(self)
    }
}

// ---------------------------------------------------------------------------
// Project configuration — `.rsopencode/project.toml`
// ---------------------------------------------------------------------------

/// Project-local configuration stored at `.rsopencode/project.toml`.
///
/// Overrides global defaults for a specific project. This file is meant to be
/// committed to version control so team members share the same settings.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectConfig {
    /// Model override for this project.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// Agent override for this project.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_agent: Option<String>,
    /// Shell override for this project.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shell: Option<String>,
    /// Project-specific instructions (file paths relative to project root).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub instructions: Vec<String>,
    /// Enabled skills for this project.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub skills: Vec<String>,
    /// Whether to enable file snapshots for this project.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snapshots: Option<bool>,
}

impl ProjectConfig {
    /// Parses a TOML string into a [`ProjectConfig`].
    pub fn from_toml_str(s: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(s)
    }

    /// Serializes to a TOML string.
    pub fn to_toml_string(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(self)
    }
}

// ---------------------------------------------------------------------------
// Session metadata — `.rsopencode/sessions/<id>.toml`
// ---------------------------------------------------------------------------

/// Session metadata stored as `.rsopencode/sessions/<id>.toml`.
///
/// Each session gets a small TOML file recording its ID, title, creation
/// time, and the model/agent used. The full transcript is stored separately
/// by the storage layer.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionMeta {
    /// Unique session identifier.
    pub id: String,
    /// Human-readable title for the session.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// ISO-8601 creation timestamp.
    pub created_at: String,
    /// Model used by the session.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// Agent used by the session.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent: Option<String>,
    /// Project directory the session belongs to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub directory: Option<String>,
}

impl SessionMeta {
    /// Serializes to a TOML string.
    pub fn to_toml_string(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(self)
    }

    /// Parses a TOML string into a [`SessionMeta`].
    pub fn from_toml_str(s: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(s)
    }
}

// ---------------------------------------------------------------------------
// Directory manager
// ---------------------------------------------------------------------------

/// Manages the `.rsopencode` directory layout for a given project.
///
/// Project-local paths are resolved relative to `project_dir`. The global
/// directory is resolved from the user's home directory.
pub struct RsOpenCodeDir {
    project_dir: PathBuf,
}

impl RsOpenCodeDir {
    /// Creates a new manager rooted at the given project directory.
    pub fn new(project_dir: impl Into<PathBuf>) -> Self {
        Self { project_dir: project_dir.into() }
    }

    /// Creates a manager rooted at the current working directory.
    pub fn from_cwd() -> io::Result<Self> {
        let cwd = std::env::current_dir()?;
        Ok(Self::new(cwd))
    }

    // -- project-local paths ----------------------------------------------

    /// Returns the project-local `.rsopencode` directory.
    pub fn project_dir(&self) -> PathBuf {
        self.project_dir.join(DIR_NAME)
    }

    /// Returns the path to `.rsopencode/project.toml`.
    pub fn project_config_path(&self) -> PathBuf {
        self.project_dir().join(PROJECT_CONFIG_FILE)
    }

    /// Returns the path to `.rsopencode/sessions/`.
    pub fn sessions_dir(&self) -> PathBuf {
        self.project_dir().join(SESSIONS_DIR)
    }

    /// Returns the path to a session metadata file.
    pub fn session_path(&self, session_id: &str) -> PathBuf {
        self.sessions_dir().join(format!("{session_id}.toml"))
    }

    // -- global paths -----------------------------------------------------

    /// Returns the global `~/.rsopencode` directory.
    pub fn global_dir(&self) -> io::Result<PathBuf> {
        let home = dirs::home_dir()
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "home directory"))?;
        Ok(home.join(DIR_NAME))
    }

    /// Returns the path to `~/.rsopencode/config.toml`.
    pub fn global_config_path(&self) -> io::Result<PathBuf> {
        Ok(self.global_dir()?.join(GLOBAL_CONFIG_FILE))
    }

    // -- initialization ----------------------------------------------------

    /// Ensures the project-local `.rsopencode` directory and sessions
    /// subdirectory exist, creating them if necessary.
    pub fn ensure_project_dirs(&self) -> io::Result<()> {
        fs::create_dir_all(self.project_dir())?;
        fs::create_dir_all(self.sessions_dir())?;
        Ok(())
    }

    /// Ensures the global `~/.rsopencode` directory exists.
    pub fn ensure_global_dir(&self) -> io::Result<()> {
        let dir = self.global_dir()?;
        fs::create_dir_all(&dir)?;
        Ok(())
    }

    // -- global config I/O ------------------------------------------------

    /// Loads the global config from `~/.rsopencode/config.toml`.
    ///
    /// Returns `Default` when the file does not exist.
    pub fn load_global_config(&self) -> io::Result<GlobalConfig> {
        let path = self.global_config_path()?;
        match fs::read_to_string(&path) {
            Ok(content) => GlobalConfig::from_toml_str(&content)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e)),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(GlobalConfig::default()),
            Err(e) => Err(e),
        }
    }

    /// Saves the global config to `~/.rsopencode/config.toml`, creating the
    /// directory if necessary.
    pub fn save_global_config(&self, config: &GlobalConfig) -> io::Result<()> {
        self.ensure_global_dir()?;
        let path = self.global_config_path()?;
        let toml = config
            .to_toml_string()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        fs::write(&path, toml)?;
        Ok(())
    }

    /// Generates a default global config file at `~/.rsopencode/config.toml`
    /// if one does not already exist.
    pub fn init_global_config(&self) -> io::Result<()> {
        let path = self.global_config_path()?;
        if path.exists() {
            return Ok(());
        }
        self.ensure_global_dir()?;
        fs::write(&path, DEFAULT_GLOBAL_CONFIG_CONTENT)
    }

    // -- project config I/O ----------------------------------------------

    /// Loads the project config from `.rsopencode/project.toml`.
    ///
    /// Returns `Default` when the file does not exist.
    pub fn load_project_config(&self) -> io::Result<ProjectConfig> {
        let path = self.project_config_path();
        match fs::read_to_string(&path) {
            Ok(content) => ProjectConfig::from_toml_str(&content)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e)),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(ProjectConfig::default()),
            Err(e) => Err(e),
        }
    }

    /// Saves the project config to `.rsopencode/project.toml`, creating the
    /// directory if necessary.
    pub fn save_project_config(&self, config: &ProjectConfig) -> io::Result<()> {
        self.ensure_project_dirs()?;
        let path = self.project_config_path();
        let toml = config
            .to_toml_string()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        fs::write(&path, toml)?;
        Ok(())
    }

    /// Generates a default project config file at `.rsopencode/project.toml`
    /// if one does not already exist.
    pub fn init_project_config(&self) -> io::Result<()> {
        let path = self.project_config_path();
        if path.exists() {
            return Ok(());
        }
        self.ensure_project_dirs()?;
        fs::write(&path, DEFAULT_PROJECT_CONFIG_CONTENT)
    }

    // -- session I/O ------------------------------------------------------

    /// Saves session metadata to `.rsopencode/sessions/<id>.toml`.
    pub fn save_session(&self, meta: &SessionMeta) -> io::Result<()> {
        self.ensure_project_dirs()?;
        let path = self.session_path(&meta.id);
        let toml = meta
            .to_toml_string()
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        fs::write(&path, toml)?;
        Ok(())
    }

    /// Loads session metadata from `.rsopencode/sessions/<id>.toml`.
    pub fn load_session(&self, session_id: &str) -> io::Result<SessionMeta> {
        let path = self.session_path(session_id);
        let content = fs::read_to_string(&path)?;
        SessionMeta::from_toml_str(&content)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    /// Lists all session metadata files in `.rsopencode/sessions/`.
    pub fn list_sessions(&self) -> io::Result<Vec<SessionMeta>> {
        let dir = self.sessions_dir();
        if !dir.exists() {
            return Ok(Vec::new());
        }
        let mut sessions = Vec::new();
        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("toml") {
                continue;
            }
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(meta) = SessionMeta::from_toml_str(&content) {
                    sessions.push(meta);
                }
            }
        }
        sessions.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(sessions)
    }

    /// Removes a session metadata file.
    pub fn remove_session(&self, session_id: &str) -> io::Result<()> {
        let path = self.session_path(session_id);
        if path.exists() {
            fs::remove_file(&path)?;
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Merged configuration
// ---------------------------------------------------------------------------

/// Effective configuration after merging global and project configs.
///
/// Project-level values take precedence over global defaults.
#[derive(Debug, Clone, Default)]
pub struct MergedConfig {
    pub model: Option<String>,
    pub default_agent: Option<String>,
    pub shell: Option<String>,
    pub username: Option<String>,
    pub snapshots: Option<bool>,
    pub locale: Option<String>,
    pub theme: Option<String>,
    pub instructions: Vec<String>,
    pub skills: Vec<String>,
}

impl RsOpenCodeDir {
    /// Loads and merges global and project configs into an effective config.
    ///
    /// Project values override global ones.
    pub fn load_merged(&self) -> io::Result<MergedConfig> {
        let global = self.load_global_config()?;
        let project = self.load_project_config()?;

        Ok(MergedConfig {
            model: project.model.or(global.model),
            default_agent: project.default_agent.or(global.default_agent),
            shell: project.shell.or(global.shell),
            username: global.username,
            snapshots: project.snapshots.or(global.snapshots),
            locale: global.locale,
            theme: global.theme,
            instructions: project.instructions,
            skills: project.skills,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct ScratchDir {
        path: PathBuf,
    }

    impl ScratchDir {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "rsopencode-test-{}-{}",
                std::process::id(),
                uuid::Uuid::new_v4()
            ));
            fs::create_dir_all(&path).unwrap();
            Self { path }
        }
    }

    impl Drop for ScratchDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    #[test]
    fn project_config_roundtrip() {
        let scratch = ScratchDir::new();
        let dir = RsOpenCodeDir::new(&scratch.path);

        let config = ProjectConfig {
            model: Some("anthropic/claude-sonnet-4-5".into()),
            default_agent: Some("build".into()),
            instructions: vec!["AGENTS.md".into()],
            ..Default::default()
        };
        dir.save_project_config(&config).unwrap();

        let loaded = dir.load_project_config().unwrap();
        assert_eq!(loaded.model.as_deref(), Some("anthropic/claude-sonnet-4-5"));
        assert_eq!(loaded.default_agent.as_deref(), Some("build"));
        assert_eq!(loaded.instructions, vec!["AGENTS.md".to_string()]);
    }

    #[test]
    fn missing_project_config_returns_default() {
        let scratch = ScratchDir::new();
        let dir = RsOpenCodeDir::new(&scratch.path);
        let config = dir.load_project_config().unwrap();
        assert!(config.model.is_none());
    }

    #[test]
    fn session_roundtrip() {
        let scratch = ScratchDir::new();
        let dir = RsOpenCodeDir::new(&scratch.path);

        let meta = SessionMeta {
            id: "sess-123".into(),
            title: Some("Test session".into()),
            created_at: "2026-01-01T00:00:00Z".into(),
            model: Some("anthropic/claude-sonnet-4-5".into()),
            agent: Some("build".into()),
            directory: Some("/tmp/project".into()),
        };
        dir.save_session(&meta).unwrap();
        let loaded = dir.load_session("sess-123").unwrap();
        assert_eq!(loaded.id, "sess-123");
        assert_eq!(loaded.title.as_deref(), Some("Test session"));
    }

    #[test]
    fn list_sessions_returns_sorted() {
        let scratch = ScratchDir::new();
        let dir = RsOpenCodeDir::new(&scratch.path);

        dir.save_session(&SessionMeta {
            id: "old".into(),
            created_at: "2026-01-01T00:00:00Z".into(),
            ..Default::default()
        })
        .unwrap();
        dir.save_session(&SessionMeta {
            id: "new".into(),
            created_at: "2026-06-01T00:00:00Z".into(),
            ..Default::default()
        })
        .unwrap();

        let sessions = dir.list_sessions().unwrap();
        assert_eq!(sessions.len(), 2);
        assert_eq!(sessions[0].id, "new");
        assert_eq!(sessions[1].id, "old");
    }

    #[test]
    fn merged_config_project_overrides_global() {
        let scratch = ScratchDir::new();
        let dir = RsOpenCodeDir::new(&scratch.path);

        let global = GlobalConfig {
            model: Some("global-model".into()),
            shell: Some("zsh".into()),
            ..Default::default()
        };
        // Save global config directly into the project dir for testing.
        fs::create_dir_all(dir.project_dir()).unwrap();
        fs::write(
            dir.project_dir().join(GLOBAL_CONFIG_FILE),
            global.to_toml_string().unwrap(),
        )
        .unwrap();

        let project = ProjectConfig {
            model: Some("project-model".into()),
            ..Default::default()
        };
        dir.save_project_config(&project).unwrap();

        let merged = dir.load_merged().unwrap();
        assert_eq!(merged.model.as_deref(), Some("project-model"));
        assert_eq!(merged.shell.as_deref(), Some("zsh"));
    }
}
