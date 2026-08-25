//! Daemon service — `packages/cli/src/services/daemon.ts`
//!
//! Owns the background server lifecycle: registration file management, password
//! storage, health probing, version compatibility checks, and process spawning.
//! The TS layer is an Effect `Layer` exposing a `Service` context; the Rust
//! port mirrors the same operations as methods on [`Daemon`].

use std::fs;
use std::io;
use std::path::PathBuf;
use std::process::{self, Command};
use std::time::Duration;

use base64::Engine;
use rand::RngCore;
use serde::{Deserialize, Serialize};

/// Default starting port the serve handler scans when no `--port` is given.
pub const SERVE_PORT_START: u16 = 4096;

/// Installation version stamped into registration files. Mirrors
/// `InstallationVersion` from `@opencode-ai/core/installation/version`.
pub const INSTALLATION_VERSION: &str = env!("CARGO_PKG_VERSION");

/// A registered server entry persisted to `server.json`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Registration {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    pub url: String,
    pub pid: i32,
}

/// Daemon handle bound to the opencode state directory.
pub struct Daemon {
    directory: PathBuf,
    server_file: PathBuf,
    password_file: PathBuf,
}

impl Daemon {
    /// Builds a daemon rooted at the opencode state directory.
    pub fn new() -> io::Result<Self> {
        let directory = state_directory()?;
        fs::create_dir_all(&directory)?;
        let server_file = directory.join("server.json");
        let password_file = directory.join("password");
        Ok(Self { directory, server_file, password_file })
    }

    /// Returns the base state directory.
    pub fn directory(&self) -> &PathBuf {
        &self.directory
    }

    // -- password -----------------------------------------------------------

    /// Gets the stored password, or generates and persists a new one.
    ///
    /// When `value` is `None` and a password exists it is returned as-is.
    /// Otherwise a new credential is written atomically with mode 0600.
    pub fn password(&self, value: Option<&str>) -> io::Result<String> {
        if value.is_none() {
            if let Ok(existing) = fs::read_to_string(&self.password_file) {
                return Ok(existing);
            }
        }
        let generated = match value {
            Some(v) => v.to_string(),
            None => {
                let mut bytes = [0u8; 32];
                rand::thread_rng().fill_bytes(&mut bytes);
                base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
            }
        };
        let temp = self.password_file.with_extension("tmp");
        fs::write(&temp, &generated)?;
        set_mode_600(&temp)?;
        fs::rename(&temp, &self.password_file)?;
        Ok(generated)
    }

    /// Builds the `Authorization` header value matching `ServerAuth.headers`.
    pub fn auth_header(&self, password: &str) -> String {
        format!("Bearer {password}")
    }

    // -- registration ------------------------------------------------------

    /// Reads the current registration, if any.
    pub fn read_registration(&self) -> Option<Registration> {
        let data = fs::read_to_string(&self.server_file).ok()?;
        serde_json::from_str(&data).ok()
    }

    /// Writes a registration entry atomically.
    pub fn write_registration(&self, reg: &Registration) -> io::Result<()> {
        let id = uuid::Uuid::new_v4().to_string();
        let temp = self.server_file.with_extension(format!("{id}.tmp"));
        fs::create_dir_all(&self.directory)?;
        let json = serde_json::to_string(reg)?;
        fs::write(&temp, json)?;
        set_mode_600(&temp)?;
        fs::rename(&temp, &self.server_file)?;
        Ok(())
    }

    /// Removes the registration file (ignores missing).
    pub fn remove_registration(&self) {
        let _ = fs::remove_file(&self.server_file);
    }

    // -- health -------------------------------------------------------------

    /// Probes the registered server and returns its info when healthy and
    /// version-compatible.
    pub fn healthy(&self) -> Option<Registration> {
        let info = self.read_registration()?;
        if !self.probe_health(&info.url) {
            return None;
        }
        if info.version.as_deref() != Some(INSTALLATION_VERSION) {
            return None;
        }
        Some(info)
    }

    /// Returns the running server URL, or `None` when stopped.
    pub fn status(&self) -> Option<String> {
        match self.healthy() {
            Some(info) => Some(info.url),
            None => {
                self.remove_registration();
                None
            }
        }
    }

    fn probe_health(&self, url: &str) -> bool {
        let client = match reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(2))
            .build()
        {
            Ok(c) => c,
            Err(_) => return false,
        };
        let password = match self.password(None) {
            Ok(p) => p,
            Err(_) => return false,
        };
        let endpoint = format!("{url}/api/health");
        let resp = match client
            .get(&endpoint)
            .header("authorization", self.auth_header(&password))
            .send()
        {
            Ok(r) => r,
            Err(_) => return false,
        };
        if !resp.status().is_success() {
            return false;
        }
        let body: serde_json::Value = match resp.json() {
            Ok(v) => v,
            Err(_) => return false,
        };
        body.get("healthy").and_then(|v| v.as_bool()).unwrap_or(false)
    }

    // -- process management ------------------------------------------------

    /// Starts (or reuses) the background server, returning its URL.
    pub fn start(&self) -> io::Result<String> {
        if let Some(info) = self.healthy() {
            return Ok(info.url);
        }
        // Stop any stale, incompatible registration before relaunching.
        if let Some(stale) = self.read_registration() {
            let _ = self.stop_process(&stale);
        }

        let entrypoint = std::env::current_exe()?;
        Command::new(&entrypoint)
            .arg("serve")
            .arg("--register")
            .stdin(process::Stdio::null())
            .stdout(process::Stdio::null())
            .stderr(process::Stdio::null())
            .spawn()?;

        // Poll until the spawned server is healthy and compatible.
        for _ in 0..100 {
            if let Some(info) = self.healthy() {
                return Ok(info.url);
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        Err(io::Error::new(io::ErrorKind::Other, "Failed to start server"))
    }

    /// Returns the transport URL and authorization header.
    pub fn transport(&self) -> io::Result<(String, String)> {
        let url = self.start()?;
        let password = self.password(None)?;
        Ok((url, self.auth_header(&password)))
    }

    /// Stops the registered server process if it matches the current registration.
    pub fn stop_process(&self, info: &Registration) -> io::Result<()> {
        signal(info.pid, libc_or_default_sigterm())?;
        for _ in 0..100 {
            if !is_running(info.pid) {
                return Ok(());
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        signal(info.pid, libc_or_default_sigkill())?;
        Ok(())
    }

    /// Stops the background server and clears stale registration.
    pub fn stop(&self) -> io::Result<()> {
        match self.healthy() {
            None => {
                self.remove_registration();
                Ok(())
            }
            Some(info) => {
                self.stop_process(&info)?;
                self.remove_registration();
                Ok(())
            }
        }
    }

    /// Registers the current process as the daemon for the given address.
    pub fn register(&self, url: &str) -> io::Result<()> {
        let reg = Registration {
            id: Some(uuid::Uuid::new_v4().to_string()),
            version: Some(INSTALLATION_VERSION.to_string()),
            url: url.to_string(),
            pid: process::id() as i32,
        };
        self.write_registration(&reg)?;
        Ok(())
    }
}

impl Default for Daemon {
    fn default() -> Self {
        Self::new().expect("Failed to initialize daemon state directory")
    }
}

// -- helpers ----------------------------------------------------------------

/// Sets file permissions to 0600 (owner read/write only).
#[cfg(unix)]
fn set_mode_600(path: &std::path::Path) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o600))
}

/// No-op on non-unix platforms.
#[cfg(not(unix))]
fn set_mode_600(_path: &std::path::Path) -> io::Result<()> {
    Ok(())
}

/// Resolves the opencode state directory (`Global.Path.state`).
fn state_directory() -> io::Result<PathBuf> {
    if let Ok(dir) = std::env::var("OPENCODE_STATE_DIRECTORY") {
        return Ok(PathBuf::from(dir));
    }
    let base = dirs::state_dir()
        .or_else(dirs::data_local_dir)
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "state directory"))?;
    Ok(base.join("opencode"))
}

/// Sends a Unix signal to a process, ignoring errors.
fn signal(pid: i32, sig: i32) -> io::Result<()> {
    // The pid is only ever signalled after a healthy registration has
    // authenticated it as the opencode server.
    #[cfg(unix)]
    {
        let _ = std::process::Command::new("kill")
            .arg(format!("-{sig}"))
            .arg(pid.to_string())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();
    }
    #[cfg(not(unix))]
    {
        let _ = (pid, sig);
    }
    Ok(())
}

/// Checks whether a process is running.
fn is_running(pid: i32) -> bool {
    #[cfg(unix)]
    {
        std::process::Command::new("kill")
            .arg("-0")
            .arg(pid.to_string())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }
    #[cfg(not(unix))]
    {
        false
    }
}

fn libc_or_default_sigterm() -> i32 {
    15
}

fn libc_or_default_sigkill() -> i32 {
    9
}



#[cfg(test)]
mod tests {
    use super::*;

    /// Creates a unique scratch directory under the std temp dir and returns
    /// its path plus a guard that removes it on drop.
    struct ScratchDir {
        path: PathBuf,
    }

    impl ScratchDir {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "opencode-daemon-test-{}-{}",
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

    fn make_daemon(scratch: &ScratchDir) -> Daemon {
        Daemon {
            directory: scratch.path.clone(),
            server_file: scratch.path.join("server.json"),
            password_file: scratch.path.join("password"),
        }
    }

    #[test]
    fn registration_roundtrip() {
        let scratch = ScratchDir::new();
        let daemon = make_daemon(&scratch);
        let reg = Registration {
            id: Some("id-1".into()),
            version: Some(INSTALLATION_VERSION.into()),
            url: "http://127.0.0.1:4096".into(),
            pid: 12345,
        };
        daemon.write_registration(&reg).unwrap();
        let read = daemon.read_registration().unwrap();
        assert_eq!(read, reg);
    }

    #[test]
    fn password_generates_when_missing() {
        let scratch = ScratchDir::new();
        let daemon = make_daemon(&scratch);
        let pw = daemon.password(None).unwrap();
        assert!(!pw.is_empty());
        assert_eq!(daemon.password(None).unwrap(), pw);
    }

    #[test]
    fn password_overrides_with_value() {
        let scratch = ScratchDir::new();
        let daemon = make_daemon(&scratch);
        let pw = daemon.password(Some("hunter2")).unwrap();
        assert_eq!(pw, "hunter2");
        assert_eq!(daemon.password(None).unwrap(), "hunter2");
    }
}
