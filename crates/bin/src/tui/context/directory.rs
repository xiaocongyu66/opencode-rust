use super::project::ProjectContext;
use super::sync::SyncContext;
use super::runtime::TuiPaths;

pub struct DirectoryContext {
    pub project: ProjectContext,
    pub sync: SyncContext,
    pub paths: TuiPaths,
}

impl DirectoryContext {
    pub fn current(&self) -> String {
        let directory = self.project.instance.directory().unwrap_or(self.paths.cwd.clone());
        let abbreviated = abbreviate_home(&directory, &self.paths.home);
        if let Some(branch) = self.sync.vcs_branch() {
            format!("{}:{}", abbreviated, branch)
        } else {
            abbreviated
        }
    }
}

pub fn abbreviate_home(path: &str, home: &str) -> String {
    if !home.is_empty() && path.starts_with(home) {
        let rest = &path[home.len()..];
        if rest.is_empty() {
            "~".to_string()
        } else {
            format!("~{}", rest)
        }
    } else {
        path.to_string()
    }
}
