//! Session tools resolution.
//!
//! Ported from `session/tools.ts`.
//! Resolves available tools for a session including MCP tools.

use std::collections::HashMap;

use crate::core::mcp::McpManager;
use crate::schema::ids::SessionID;
use crate::schema::session::SessionMessage;

/// Max blob size for MCP resource attachments.
pub const MAX_MCP_RESOURCE_BLOB_BYTES: usize = 10 * 1024 * 1024;

/// Supported MIME types for MCP resource attachments.
pub const SUPPORTED_MCP_RESOURCE_ATTACHMENT_MIMES: &[&str] = &[
    "application/pdf",
    "image/gif",
    "image/jpeg",
    "image/png",
    "image/webp",
];

/// MCP resource tool names.
pub const MCP_RESOURCE_TOOLS_LIST: &str = "list_mcp_resources";
pub const MCP_RESOURCE_TOOLS_LIST_TEMPLATES: &str = "list_mcp_resource_templates";
pub const MCP_RESOURCE_TOOLS_READ: &str = "read_mcp_resource";

/// Tool context — passed to each tool execution.
#[derive(Debug, Clone)]
pub struct ToolContext {
    pub session_id: SessionID,
    pub message_id: String,
    pub call_id: String,
    pub agent: String,
    pub messages: Vec<SessionMessage>,
}

/// Resolve all available tools for a session.
pub async fn resolve_tools(
    _mcp: &McpManager,
    _session_id: &SessionID,
    _agent: &str,
) -> HashMap<String, serde_json::Value> {
    let tools = HashMap::new();
    // Built-in tools would be registered here
    // MCP tools would be added here
    tools
}

/// Calculate the byte size of a base64-encoded blob.
pub fn base64_size(value: &str) -> usize {
    let trimmed: String = value.chars().filter(|c| !c.is_whitespace()).collect();
    let padding = if trimmed.ends_with("==") {
        2
    } else if trimmed.ends_with('=') {
        1
    } else {
        0
    };
    let len = trimmed.len();
    if len == 0 {
        return 0;
    }
    ((len * 3) / 4).saturating_sub(padding)
}

/// Format bytes as human-readable string.
pub fn format_bytes(value: usize) -> String {
    if value < 1024 {
        format!("{} B", value)
    } else if value < 1024 * 1024 {
        format!("{} KB", (value as f64 / 1024.0).ceil() as usize)
    } else {
        format!("{} MB", (value as f64 / (1024.0 * 1024.0)).ceil() as usize)
    }
}
