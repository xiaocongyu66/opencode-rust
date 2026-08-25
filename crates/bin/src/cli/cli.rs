//! CLI command definitions — `packages/cli/src/commands/commands.ts`
//!
//! Mirrors the TypeScript command spec built on `effect/unstable/cli`. The
//! Rust port uses `clap` derive with the same command tree, flag names, and
//! defaults so the two CLIs stay behaviourally equivalent.

use clap::{Parser, Subcommand};

/// OpenCode 2.0 preview command line interface.
#[derive(Parser)]
#[command(
    name = "rsopencode",
    about = "OpenCode 2.0 preview command line interface"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

/// Top-level commands registered in `commands.ts`.
#[derive(Subcommand)]
pub enum Commands {
    /// Start interactive terminal session (default handler `$`).
    #[command(alias = "default", alias = "chat")]
    Default {
        /// Resume a previous session by id (e.g. `rsopencode --resume ses_xxx`).
        #[arg(long, value_name = "SESSION_ID")]
        resume: Option<String>,
    },

    /// Make a request to the running server.
    Api {
        /// OpenAPI operation ID, or an HTTP method followed by a path.
        ///
        /// Variadic 1–2 args: `operation` | `method path`.
        #[arg(num_args = 1..=2, required = true)]
        request: Vec<String>,

        /// Request body.
        #[arg(short = 'd', long)]
        data: Option<String>,

        /// Request header in `name:value` form (repeatable, up to 100).
        #[arg(short = 'H', long = "header", num_args = 1, action = clap::ArgAction::Append)]
        header: Vec<String>,

        /// OpenAPI path or query parameter (`key=value`, repeatable).
        #[arg(long = "param", num_args = 1, value_parser = parse_key_value, action = clap::ArgAction::Append)]
        param: Vec<(String, String)>,
    },

    /// Debugging and troubleshooting tools.
    Debug {
        #[command(subcommand)]
        action: DebugAction,
    },

    /// Migrate v1 data to v2.
    Migrate,

    /// Manage the background server.
    Service {
        #[command(subcommand)]
        action: ServiceAction,
    },

    /// Start the v2 API server.
    Serve {
        /// Bind hostname.
        #[arg(long, default_value = "127.0.0.1")]
        hostname: String,

        /// Bind port. When omitted, the server scans from 4096 upwards.
        #[arg(long)]
        port: Option<u16>,

        /// Register this server as the background daemon.
        #[arg(long, default_value_t = false)]
        register: bool,
    },

    /// Check for a newer release and self-update.
    Update {
        /// Print the target version without installing.
        #[arg(long, default_value_t = false)]
        check: bool,
    },
}

/// `debug` subcommands.
#[derive(Subcommand)]
pub enum DebugAction {
    /// List all agents.
    Agents,
}

/// `service` subcommands.
#[derive(Subcommand)]
pub enum ServiceAction {
    /// Start the background server.
    Start,
    /// Restart the background server.
    Restart,
    /// Show background server status.
    Status,
    /// Stop the background server.
    Stop,
    /// Get or set the server password.
    Password {
        /// When provided, replaces the stored password (stops the server first).
        value: Option<String>,
    },
}

/// Parses `key=value` pairs for the `--param` flag.
fn parse_key_value(input: &str) -> Result<(String, String), String> {
    let idx = input
        .find('=')
        .ok_or_else(|| format!("expected key=value, got: {input}"))?;
    Ok((input[..idx].to_string(), input[idx + 1..].to_string()))
}
