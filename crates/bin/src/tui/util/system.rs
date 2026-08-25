use std::env;

pub fn describe_os() -> String {
    let name = if cfg!(target_os = "macos") {
        "macOS"
    } else if cfg!(target_os = "windows") {
        "Windows"
    } else if cfg!(target_os = "linux") {
        "Linux"
    } else {
        std::env::consts::OS
    };
    let arch = std::env::consts::ARCH;
    let release = SystemRelease::get();
    format!("{} {} ({})", name, release, arch)
}

pub fn describe_terminal() -> String {
    let program = env::var("TERM_PROGRAM")
        .or_else(|_| env::var("TERM"))
        .unwrap_or_else(|_| "unknown".to_string());
    let version = env::var("TERM_PROGRAM_VERSION")
        .map(|v| format!(" {}", v))
        .unwrap_or_default();
    let multiplexer = if env::var("TMUX").is_ok() {
        " in tmux"
    } else if env::var("STY").is_ok() {
        " in screen"
    } else {
        ""
    };
    format!("{}{}{}", program, version, multiplexer)
}

struct SystemRelease;

impl SystemRelease {
    fn get() -> String {
        // Use `uname -r` on unix-like systems; no external crate needed.
        #[cfg(unix)]
        {
            use std::process::Command;
            Command::new("uname")
                .arg("-r")
                .output()
                .ok()
                .and_then(|o| String::from_utf8(o.stdout).ok())
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| "unknown".to_string())
        }
        #[cfg(windows)]
        {
            env::var("OSVER").unwrap_or_else(|_| "unknown".to_string())
        }
        #[cfg(not(any(unix, windows)))]
        {
            "unknown".to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_describe_os() {
        let os = describe_os();
        assert!(!os.is_empty());
        assert!(os.contains("macOS") || os.contains("Linux") || os.contains("Windows"));
    }

    #[test]
    fn test_describe_terminal() {
        let term = describe_terminal();
        assert!(!term.is_empty());
    }
}
