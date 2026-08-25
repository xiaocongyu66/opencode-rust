//! Session instruction resolution.
//!
//! Ported from `session/instruction.ts`.
//! Resolves AGENTS.md, CLAUDE.md, and other instruction files.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

/// Instruction file names to search for.
pub const INSTRUCTION_FILES: &[&str] = &["AGENTS.md", "CLAUDE.md", "CONTEXT.md"];

/// Instruction manager — resolves instruction files from project and global config.
pub struct InstructionManager {
    global_files: Vec<PathBuf>,
    worktree: PathBuf,
    directory: PathBuf,
    claims: HashMap<String, HashSet<PathBuf>>,
}

impl InstructionManager {
    pub fn new(global_config: &str, home: &str, worktree: &str, directory: &str) -> Self {
        let mut global_files = vec![PathBuf::from(global_config).join("AGENTS.md")];
        global_files.push(PathBuf::from(home).join(".claude").join("CLAUDE.md"));

        Self {
            global_files,
            worktree: PathBuf::from(worktree),
            directory: PathBuf::from(directory),
            claims: HashMap::new(),
        }
    }

    /// Get all system instruction paths.
    pub fn system_paths(&self) -> Vec<PathBuf> {
        let mut paths = HashSet::new();

        for file in &self.global_files {
            if file.exists() {
                paths.insert(file.clone());
                break;
            }
        }

        for file in INSTRUCTION_FILES {
            let found = self.find_up(file, &self.directory, &self.worktree);
            if !found.is_empty() {
                for f in found {
                    paths.insert(f);
                }
                break;
            }
        }

        paths.into_iter().collect()
    }

    /// Read all system instructions.
    pub fn system(&self) -> Vec<String> {
        let paths = self.system_paths();
        let mut results = Vec::new();
        for path in paths {
            if let Ok(content) = std::fs::read_to_string(&path) {
                results.push(format!("Instructions from: {}\n{}", path.display(), content));
            }
        }
        results
    }

    /// Find an instruction file in a specific directory.
    pub fn find(&self, dir: &str) -> Option<PathBuf> {
        for file in INSTRUCTION_FILES {
            let filepath = Path::new(dir).join(file);
            if filepath.exists() {
                return Some(filepath);
            }
        }
        None
    }

    /// Clear instruction claims for a message.
    pub fn clear(&mut self, message_id: &str) {
        self.claims.remove(message_id);
    }

    /// Resolve instruction files near a file being read.
    pub fn resolve(
        &mut self,
        filepath: &str,
        message_id: &str,
    ) -> Vec<(PathBuf, String)> {
        let sys = self.system_paths();
        let target = PathBuf::from(filepath);
        let root = self.worktree.canonicalize().unwrap_or(self.worktree.clone());
        let mut current = target
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| self.directory.clone());

        let mut results = Vec::new();

        while current.starts_with(&root) && current != root {
            if let Some(found) = self.find(&current.to_string_lossy()) {
                if found == target || sys.contains(&found) {
                    current = current
                        .parent()
                        .map(|p| p.to_path_buf())
                        .unwrap_or_else(|| current.clone());
                    continue;
                }

                let claims = self.claims.entry(message_id.to_string()).or_default();
                if claims.contains(&found) {
                    current = current
                        .parent()
                        .map(|p| p.to_path_buf())
                        .unwrap_or_else(|| current.clone());
                    continue;
                }

                claims.insert(found.clone());

                if let Ok(content) = std::fs::read_to_string(&found) {
                    if !content.is_empty() {
                        results.push((
                            found.clone(),
                            format!("Instructions from: {}\n{}", found.display(), content),
                        ));
                    }
                }
            }

            current = current
                .parent()
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|| current.clone());
        }

        results
    }

    fn find_up(&self, file: &str, start: &Path, root: &Path) -> Vec<PathBuf> {
        let mut results = Vec::new();
        let mut current = start.to_path_buf();
        while current.starts_with(root) {
            let candidate = current.join(file);
            if candidate.exists() {
                results.push(candidate);
                break;
            }
            if current == root {
                break;
            }
            current = match current.parent() {
                Some(p) => p.to_path_buf(),
                None => break,
            };
        }
        results
    }
}

/// Extract loaded file paths from tool results in messages.
pub fn extract_loaded_files(messages: &[crate::schema::session::SessionMessage]) -> HashSet<String> {
    let mut paths = HashSet::new();
    for msg in messages {
        if let crate::schema::session::SessionMessage::Assistant { content, .. } = msg {
            for item in content {
                if let crate::schema::session::AssistantContent::Tool { name, state, .. } = item {
                    if name == "read" {
                        if let crate::schema::session::ToolState::Completed { structured, .. } = state {
                            if let Some(loaded) = structured.get("loaded") {
                                if let Some(arr) = loaded.as_array() {
                                    for item in arr {
                                        if let Some(s) = item.as_str() {
                                            paths.insert(s.to_string());
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    paths
}
