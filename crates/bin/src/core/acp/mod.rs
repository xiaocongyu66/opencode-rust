//! ACP — Agent Client Protocol layer.
//!
//! Decouples agent core (runner) from frontends (TUI/print/IDE) per
//! claude-code-book Ch02/Ch13. The runner emits RunnerEvent via mpsc;
//! AcpBridge converts them to AcpEvent which frontends subscribe to.

pub mod bridge;
pub mod protocol;

pub use bridge::spawn_bridge;
pub use protocol::{from_runner_event, AcpEvent, StreamDelta};
