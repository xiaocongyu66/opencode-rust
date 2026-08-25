use super::directory::abbreviate_home;
use super::location::LocationContext;
use super::runtime::TuiPaths;
use std::path::{Path, PathBuf};

pub struct PathFormatContext {
    pub paths: TuiPaths,
    pub location: LocationContext,
}

impl PathFormatContext {
    pub fn path(&self) -> String {
        self.location
            .get()
            .map(|l| l.directory)
            .unwrap_or_else(|| self.paths.cwd.clone())
    }

    pub fn format(&self, input: Option<&str>) -> String {
        match input {
            None => String::new(),
            Some(s) if s.is_empty() => String::new(),
            Some(input) => {
                let base = self.path();
                format_path(input, &base, &self.paths.home)
            }
        }
    }
}

fn format_path(input: &str, base: &str, home: &str) -> String {
    let absolute = if Path::new(input).is_absolute() {
        PathBuf::from(input)
    } else {
        PathBuf::from(base).join(input)
    };

    let absolute = absolute.canonicalize().unwrap_or(absolute);

    let relative = Path::new(base).canonicalize().unwrap_or_else(|_| PathBuf::from(base));
    let relative = absolute.strip_prefix(&relative).map(|p| p.to_string_lossy().to_string());

    match relative {
        Ok(rel) if rel.is_empty() => ".".to_string(),
        Ok(rel) if rel != ".." && !rel.starts_with("..") => rel,
        _ => abbreviate_home(&absolute.to_string_lossy(), home),
    }
}
