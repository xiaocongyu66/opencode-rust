//! CLI services — `packages/cli/src/services/`
//!
//! Background daemon management and supporting services.

pub mod daemon;

pub use daemon::{Daemon, Registration, INSTALLATION_VERSION, SERVE_PORT_START};
