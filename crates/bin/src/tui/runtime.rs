//! TUI runtime — paths, terminal environment, and window control.
//! Ported from tui/src/runtime.tsx + tui/src/terminal-win32.ts + tui/src/clipboard.ts

use std::env;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

/// TUI filesystem paths — mirrors `TuiPathsProvider` context value.
#[derive(Debug, Clone)]
pub struct TuiPaths {
    pub cwd: PathBuf,
    pub home: PathBuf,
    pub state: PathBuf,
    pub worktree: PathBuf,
}

impl TuiPaths {
    pub fn from_global(home: &Path, state: &Path, data: &Path) -> Self {
        Self {
            cwd: env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            home: home.to_path_buf(),
            state: state.to_path_buf(),
            worktree: data.join("worktree"),
        }
    }
}

/// Terminal multiplexer and display server detection.
#[derive(Debug, Clone)]
pub struct TerminalEnvironment {
    pub platform: String,
    pub multiplexer: Option<String>,
    pub display_server: Option<String>,
}

impl TerminalEnvironment {
    pub fn detect() -> Self {
        let platform = env::consts::OS.to_string();
        let multiplexer = if env::var("TMUX").is_ok() {
            Some("tmux".to_string())
        } else if env::var("STY").is_ok() {
            Some("screen".to_string())
        } else {
            None
        };
        let display_server = if env::var("WAYLAND_DISPLAY").is_ok() {
            Some("wayland".to_string())
        } else if env::var("DISPLAY").is_ok() {
            Some("x11".to_string())
        } else {
            None
        };
        Self {
            platform,
            multiplexer,
            display_server,
        }
    }
}

/// Startup configuration — mirrors `TuiStartupProvider` context value.
#[derive(Debug, Clone)]
pub struct TuiStartup {
    pub initial_route: Option<String>,
    pub skip_initial_loading: bool,
}

impl TuiStartup {
    pub fn from_env() -> Self {
        let initial_route = env::var("OPENCODE_ROUTE").ok();
        let skip_initial_loading = env::var("OPENCODE_FAST_BOOT").is_ok();
        Self {
            initial_route,
            skip_initial_loading,
        }
    }
}

/// Abbreviate home directory to `~` — mirrors `abbreviateHome` from runtime.tsx.
pub fn abbreviate_home(input: &str, home: &str) -> String {
    if home.is_empty() {
        return input.to_string();
    }
    let home_path = Path::new(home);
    let input_path = Path::new(input);
    match input_path.strip_prefix(home_path) {
        Ok(relative) if relative.as_os_str().is_empty() => "~".to_string(),
        Ok(relative) => {
            let rel_str = relative.to_string_lossy();
            if rel_str.starts_with("..") || Path::new(rel_str.as_ref()).is_absolute() {
                input.to_string()
            } else {
                format!("~/{}", rel_str)
            }
        }
        Err(_) => input.to_string(),
    }
}

/// Set the terminal window title via OSC escape sequence.
pub fn set_terminal_title(title: &str) {
    let mut stdout = io::stdout();
    let _ = write!(stdout, "\x1b]2;{}\x07", title);
    let _ = stdout.flush();
}

/// Clear the terminal window title.
pub fn clear_terminal_title() {
    set_terminal_title("");
}

/// OSC 52 clipboard write — works inside tmux/screen passthrough.
pub fn write_osc52(text: &str) {
    let mut stdout = io::stdout();
    if let Ok(b64) = base64_encode(text.as_bytes()) {
        let sequence = format!("\x1b]52;c;{}\x07", b64);
        if env::var("TMUX").is_ok() {
            let passthrough = format!("\x1bPtmux;\x1b{}\x1b\\", sequence);
            let _ = write!(stdout, "{}{}", sequence, passthrough);
        } else if env::var("STY").is_ok() {
            let passthrough = format!("\x1bPtmux;\x1b{}\x1b\\", sequence);
            let _ = write!(stdout, "{}", passthrough);
        } else {
            let _ = write!(stdout, "{}", sequence);
        }
        let _ = stdout.flush();
    }
}

/// Simple base64 encoder (avoids adding a dependency).
fn base64_encode(input: &[u8]) -> Result<String, ()> {
    const TABLE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::with_capacity((input.len() + 2) / 3 * 4);
    for chunk in input.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let triple = (b0 << 16) | (b1 << 8) | b2;
        result.push(TABLE[((triple >> 18) & 0x3F) as usize] as char);
        result.push(TABLE[((triple >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            result.push(TABLE[((triple >> 6) & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
        if chunk.len() > 2 {
            result.push(TABLE[(triple & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
    }
    Ok(result)
}

/// Determine the native copy command for the current platform.
pub fn copy_command(os: &str, wayland: bool, has: impl Fn(&str) -> bool) -> Option<Vec<&'static str>> {
    if os == "macos" && has("osascript") {
        return Some(vec!["osascript"]);
    }
    if os == "linux" && wayland && has("wl-copy") {
        return Some(vec!["wl-copy"]);
    }
    if os == "linux" && has("xclip") {
        return Some(vec!["xclip", "-selection", "clipboard"]);
    }
    if os == "linux" && has("xsel") {
        return Some(vec!["xsel", "--clipboard", "--input"]);
    }
    if os == "windows" && has("powershell.exe") {
        return Some(vec![
            "powershell.exe",
            "-NonInteractive",
            "-NoProfile",
            "-Command",
            "[Console]::InputEncoding = [System.Text.Encoding]::UTF8; Set-Clipboard -Value ([Console]::In.ReadToEnd())",
        ]);
    }
    None
}

/// Clipboard write using OSC 52 plus a native command fallback.
pub fn clipboard_write(text: &str) {
    write_osc52(text);
    let os = env::consts::OS;
    let wayland = env::var("WAYLAND_DISPLAY").is_ok();
    if let Some(cmd) = copy_command(os, wayland, |name| which(name).is_some()) {
        use std::process::{Command, Stdio};
        use std::io::Write as IoWrite;
        let child = Command::new(cmd[0])
            .args(&cmd[1..])
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn();
        if let Ok(mut child) = child {
            if let Some(stdin) = child.stdin.as_mut() {
                let _ = stdin.write_all(text.as_bytes());
            }
            let _ = child.wait();
        }
    }
}

/// Check if a command exists in PATH.
fn which(name: &str) -> Option<PathBuf> {
    let path = env::var("PATH").ok()?;
    for dir in env::split_paths(&path) {
        let candidate = dir.join(name);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

/// Suspend the terminal process (SIGTSTP) — mirrors `terminal.suspend` command.
#[cfg(unix)]
pub fn suspend_terminal() {
    use std::process;
    let _ = process::Command::new("kill")
        .arg("-TSTP")
        .arg("0")
        .spawn();
}

/// Win32: clear ENABLE_PROCESSED_INPUT on the console stdin handle.
/// No-op on non-Windows platforms.
pub fn win32_disable_processed_input() {
    #[cfg(target_os = "windows")]
    {
        // On Windows we would call kernel32.dll via winapi.
        // For now this is a no-op stub.
    }
}

/// Win32: flush console input buffer.
/// No-op on non-Windows platforms.
pub fn win32_flush_input_buffer() {
    #[cfg(target_os = "windows")]
    {
        // On Windows we would call FlushConsoleInputBuffer via kernel32.dll.
        // For now this is a no-op stub.
    }
}

/// Renderer lifecycle: destroy and cleanup.
#[derive(Debug)]
pub struct RuntimeState {
    pub destroyed: bool,
    pub title_enabled: bool,
}

impl RuntimeState {
    pub fn new() -> Self {
        Self {
            destroyed: false,
            title_enabled: true,
        }
    }

    pub fn destroy(&mut self) {
        if self.destroyed {
            return;
        }
        clear_terminal_title();
        self.destroyed = true;
    }
}

impl Default for RuntimeState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_abbreviate_home_exact() {
        assert_eq!(abbreviate_home("/home/user", "/home/user"), "~");
    }

    #[test]
    fn test_abbreviate_home_subdir() {
        assert_eq!(abbreviate_home("/home/user/projects", "/home/user"), "~/projects");
    }

    #[test]
    fn test_abbreviate_home_other() {
        assert_eq!(abbreviate_home("/etc/config", "/home/user"), "/etc/config");
    }

    #[test]
    fn test_abbreviate_home_empty_home() {
        assert_eq!(abbreviate_home("/foo", ""), "/foo");
    }

    #[test]
    fn test_base64_encode() {
        assert_eq!(base64_encode(b"hello").unwrap(), "aGVsbG8=");
        assert_eq!(base64_encode(b"").unwrap(), "");
        assert_eq!(base64_encode(b"hi").unwrap(), "aGk=");
        assert_eq!(base64_encode(b"abc").unwrap(), "YWJj");
    }

    #[test]
    fn test_runtime_state_lifecycle() {
        let mut state = RuntimeState::new();
        assert!(!state.destroyed);
        state.destroy();
        assert!(state.destroyed);
        state.destroy();
        assert!(state.destroyed);
    }

    #[test]
    fn test_terminal_env_detect() {
        let env = TerminalEnvironment::detect();
        assert!(!env.platform.is_empty());
    }

    #[test]
    fn test_copy_command_linux_xclip() {
        let cmd = copy_command("linux", false, |_| true);
        assert_eq!(cmd, Some(vec!["xclip", "-selection", "clipboard"]));
    }

    #[test]
    fn test_copy_command_linux_wayland() {
        let cmd = copy_command("linux", true, |_| true);
        assert_eq!(cmd, Some(vec!["wl-copy"]));
    }

    #[test]
    fn test_copy_command_macos() {
        let cmd = copy_command("macos", false, |_| true);
        assert_eq!(cmd, Some(vec!["osascript"]));
    }

    #[test]
    fn test_copy_command_not_found() {
        let cmd = copy_command("linux", false, |_| false);
        assert_eq!(cmd, None);
    }

    #[test]
    fn test_tui_startup_from_env() {
        // Without env vars set, should be defaults
        let startup = TuiStartup::from_env();
        let _ = startup.skip_initial_loading;
    }
}
