//! IDE integration.
//!
//! Ported from `ide/index.ts`.
//! Detects installed IDEs and manages extension installation.

use std::process::Command;

/// Supported IDEs.
#[derive(Debug, Clone)]
pub struct SupportedIde {
    pub name: &'static str,
    pub cmd: &'static str,
}

pub const SUPPORTED_IDES: &[SupportedIde] = &[
    SupportedIde { name: "Windsurf", cmd: "windsurf" },
    SupportedIde { name: "Visual Studio Code - Insiders", cmd: "code-insiders" },
    SupportedIde { name: "Visual Studio Code", cmd: "code" },
    SupportedIde { name: "Cursor", cmd: "cursor" },
    SupportedIde { name: "VSCodium", cmd: "codium" },
];

/// Detect the current IDE from environment.
pub fn detect() -> &'static str {
    if std::env::var("TERM_PROGRAM").as_deref() == Ok("vscode") {
        if let Ok(git_askpass) = std::env::var("GIT_ASKPASS") {
            for ide in SUPPORTED_IDES {
                if git_askpass.contains(ide.name) {
                    return ide.name;
                }
            }
        }
    }
    "unknown"
}

/// Check if the extension is already installed.
pub fn already_installed() -> bool {
    matches!(
        std::env::var("OPENCODE_CALLER").as_deref(),
        Ok("vscode") | Ok("vscode-insiders")
    )
}

/// Install the opencode extension for an IDE.
pub fn install(ide_name: &str) -> Result<(), InstallError> {
    let cmd = SUPPORTED_IDES
        .iter()
        .find(|i| i.name == ide_name)
        .map(|i| i.cmd)
        .ok_or_else(|| InstallError::UnknownIde(ide_name.to_string()))?;

    let output = Command::new(cmd)
        .args(["--install-extension", "sst-dev.opencode"])
        .output()
        .map_err(|e| InstallError::Failed(e.to_string()))?;

    if !output.status.success() {
        return Err(InstallError::Failed(
            String::from_utf8_lossy(&output.stderr).to_string(),
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    if stdout.contains("already installed") {
        return Err(InstallError::AlreadyInstalled);
    }

    Ok(())
}

/// Install error.
#[derive(Debug, thiserror::Error)]
pub enum InstallError {
    #[error("Unknown IDE: {0}")]
    UnknownIde(String),
    #[error("Extension already installed")]
    AlreadyInstalled,
    #[error("Install failed: {0}")]
    Failed(String),
}
