//! Shell management.

pub struct Shell;

impl Shell {
    pub fn default_shell() -> &'static str {
        if cfg!(windows) {
            std::env::var("COMSPEC").unwrap_or_else(|_| "cmd.exe".to_string()).leak()
        } else {
            "/bin/sh"
        }
    }

    pub async fn execute(command: &str, cwd: Option<&str>) -> Result<String, std::io::Error> {
        let shell = Self::default_shell();
        let flag = if cfg!(windows) { "/C" } else { "-c" };
        let mut cmd = tokio::process::Command::new(shell);
        cmd.arg(flag).arg(command);
        if let Some(dir) = cwd {
            cmd.current_dir(dir);
        }
        let output = cmd.output().await?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.is_empty() {
            Ok(stdout.to_string())
        } else {
            Ok(format!("{}\n{}", stdout, stderr))
        }
    }
}
