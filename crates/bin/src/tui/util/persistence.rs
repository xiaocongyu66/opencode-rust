use std::collections::HashMap;
use std::path::Path;

use serde_json::Value;

pub fn read_text(file_path: &str) -> std::io::Result<String> {
    std::fs::read_to_string(file_path)
}

pub fn read_json<T: serde::de::DeserializeOwned>(file_path: &str) -> std::io::Result<T> {
    let data = std::fs::read_to_string(file_path)?;
    serde_json::from_str(&data).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
}

pub fn write_text(file_path: &str, content: &str) -> std::io::Result<()> {
    if let Some(parent) = Path::new(file_path).parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(file_path, content)
}

pub fn append_text(file_path: &str, content: &str) -> std::io::Result<()> {
    if let Some(parent) = Path::new(file_path).parent() {
        std::fs::create_dir_all(parent)?;
    }
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(file_path)?;
    file.write_all(content.as_bytes())
}

pub fn write_json_atomic(file_path: &str, value: &Value) -> std::io::Result<()> {
    if let Some(parent) = Path::new(file_path).parent() {
        std::fs::create_dir_all(parent)?;
    }
    let pid = std::process::id();
    let temp_path = format!("{}.{}.{}.tmp", file_path, pid, uuid_str());
    if let Err(e) = std::fs::write(&temp_path, serde_json::to_vec(value)?) {
        let _ = std::fs::remove_file(&temp_path);
        return Err(e);
    }
    if let Err(e) = std::fs::rename(&temp_path, file_path) {
        let _ = std::fs::remove_file(&temp_path);
        return Err(e);
    }
    Ok(())
}

fn uuid_str() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{:x}{:x}", now.as_nanos(), now.subsec_nanos())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tempfile::NamedTempFile;

    #[test]
    fn test_write_and_read_text() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path().to_str().unwrap();
        write_text(path, "hello").unwrap();
        assert_eq!(read_text(path).unwrap(), "hello");
    }

    #[test]
    fn test_append_text() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path().to_str().unwrap();
        write_text(path, "hello").unwrap();
        append_text(path, " world").unwrap();
        assert_eq!(read_text(path).unwrap(), "hello world");
    }

    #[test]
    fn test_write_json_atomic() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path().to_str().unwrap();
        let value = json!({"key": "value"});
        write_json_atomic(path, &value).unwrap();
        let result: Value = read_json(path).unwrap();
        assert_eq!(result, value);
    }
}
