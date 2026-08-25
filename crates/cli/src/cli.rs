//! CLI definition using clap.
use rust_i18n::t;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "opencode", about = t!("cli.description").to_string())]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Start the TUI session (default)
    #[command(about = t!("cli.serve").to_string())]
    Serve {
        #[arg(long, default_value = "127.0.0.1:3000")]
        addr: String,
    },

    /// Manage the background server
    Service {
        #[command(subcommand)]
        action: ServiceAction,
    },

    /// List all agents
    Agents,

    /// Make a request to the running server
    Api {
        #[arg(long)]
        method: Option<String>,
        #[arg(long)]
        path: String,
        #[arg(long)]
        data: Option<String>,
    },

    /// Migrate v1 data to v2
    Migrate,

    /// Debugging and troubleshooting tools
    Debug,
}

#[derive(Subcommand)]
pub enum ServiceAction {
    /// Start the background server
    Start,
    /// Restart the background server
    Restart,
    /// Show background server status
    Status,
    /// Stop the background server
    Stop,
}
