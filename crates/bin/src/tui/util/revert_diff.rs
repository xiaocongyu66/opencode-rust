#[derive(Debug, Clone)]
pub struct RevertDiffFile {
    pub filename: String,
    pub additions: usize,
    pub deletions: usize,
}

pub fn get_revert_diff_files(diff_text: &str) -> Vec<RevertDiffFile> {
    if diff_text.is_empty() {
        return Vec::new();
    }

    let mut result = Vec::new();
    for file_block in diff_text.split("\n+++ ") {
        if file_block.is_empty() {
            continue;
        }
        let (filename, hunks) = parse_file_block(file_block);
        if filename.is_none() {
            continue;
        }
        let (additions, deletions) = count_hunks(hunks);
        result.push(RevertDiffFile {
            filename: clean_filename(&filename.unwrap()),
            additions,
            deletions,
        });
    }
    result
}

fn parse_file_block(block: &str) -> (Option<String>, &str) {
    let first_line = block.lines().next().unwrap_or("");
    let filename = if first_line.starts_with("--- ") {
        let name = first_line.trim_start_matches("--- ").trim();
        Some(name.to_string())
    } else if first_line.starts_with("diff --git") {
        let parts: Vec<&str> = first_line.split_whitespace().collect();
        if parts.len() >= 4 {
            Some(parts[3].to_string())
        } else {
            None
        }
    } else {
        None
    };
    let hunk_start = block.find("@@").unwrap_or(block.len());
    (filename, &block[hunk_start..])
}

fn clean_filename(name: &str) -> String {
    let name = if name == "/dev/null" {
        "unknown"
    } else {
        name
    };
    let name = name.strip_prefix("a/").or_else(|| name.strip_prefix("b/")).unwrap_or(name);
    name.to_string()
}

fn count_hunks(hunks: &str) -> (usize, usize) {
    let mut additions = 0;
    let mut deletions = 0;
    for line in hunks.lines() {
        if line.starts_with('+') && !line.starts_with("+++") {
            additions += 1;
        } else if line.starts_with('-') && !line.starts_with("---") {
            deletions += 1;
        }
    }
    (additions, deletions)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty() {
        assert!(get_revert_diff_files("").is_empty());
    }

    #[test]
    fn test_single_file() {
        let diff = "--- a/foo.rs\n+++ b/foo.rs\n@@ -1,3 +1,4 @@\n-old line\n+new line\n+another\n";
        let files = get_revert_diff_files(diff);
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].filename, "foo.rs");
        assert_eq!(files[0].additions, 2);
        assert_eq!(files[0].deletions, 1);
    }

    #[test]
    fn test_multiple_files() {
        let diff = "--- a/file1.rs\n+++ b/file1.rs\n@@ -1,2 +1,2 @@\n-old\n+new\n--- a/file2.rs\n+++ b/file2.rs\n@@ -1,2 +1,2 @@\n-gone\n+here\n";
        let files = get_revert_diff_files(diff);
        assert_eq!(files.len(), 2);
    }
}
