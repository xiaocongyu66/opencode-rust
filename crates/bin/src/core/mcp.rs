//! MCP (Model Context Protocol) integration.
//!
//! Ported from `mcp/index.ts`.
//! Manages MCP client connections, tool discovery, and resource access.

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;

/// Default timeout for MCP operations.
pub const DEFAULT_TIMEOUT: u64 = 30_000;

/// MCP resource.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct McpResource {
    pub name: String,
    pub uri: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    pub client: String,
}

/// MCP server status.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum McpServerStatus {
    Connected,
    Disabled,
    Failed { error: String },
    NeedsAuth,
    NeedsClientRegistration { error: String },
}

/// MCP server info.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct McpServer {
    pub name: String,
    pub status: McpServerStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
}

/// MCP tool definition.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct McpToolDef {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<serde_json::Value>,
}

/// MCP tool with client info.
#[derive(Debug, Clone)]
pub struct McpTool {
    pub def: McpToolDef,
    pub client: String,
    pub timeout: u64,
}

/// MCP resource content.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct McpResourceContent {
    pub contents: Vec<serde_json::Value>,
}

/// MCP manager — coordinates connections to MCP servers.
pub struct McpManager {
    servers: Arc<RwLock<HashMap<String, McpServer>>>,
    tools: Arc<RwLock<HashMap<String, McpTool>>>,
    resources: Arc<RwLock<Vec<McpResource>>>,
}

impl McpManager {
    pub fn new() -> Self {
        Self {
            servers: Arc::new(RwLock::new(HashMap::new())),
            tools: Arc::new(RwLock::new(HashMap::new())),
            resources: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn list_servers(&self) -> Vec<McpServer> {
        self.servers.read().await.values().cloned().collect()
    }

    pub async fn get_server(&self, name: &str) -> Option<McpServer> {
        self.servers.read().await.get(name).cloned()
    }

    pub async fn add_server(&self, server: McpServer) {
        self.servers
            .write()
            .await
            .insert(server.name.clone(), server);
    }

    pub async fn remove_server(&self, name: &str) {
        self.servers.write().await.remove(name);
    }

    pub async fn list_tools(&self) -> HashMap<String, McpTool> {
        self.tools.read().await.clone()
    }

    pub async fn add_tool(&self, key: String, tool: McpTool) {
        self.tools.write().await.insert(key, tool);
    }

    pub async fn list_resources(&self) -> Vec<McpResource> {
        self.resources.read().await.clone()
    }

    pub async fn read_resource(&self, _server: &str, _uri: &str) -> Option<McpResourceContent> {
        None
    }

    /// Get instructions from all connected MCP servers.
    pub async fn instructions(&self) -> Vec<McpInstruction> {
        let servers = self.servers.read().await;
        servers
            .values()
            .filter_map(|s| {
                s.instructions.as_ref().map(|instructions| McpInstruction {
                    name: s.name.clone(),
                    instructions: instructions.clone(),
                    tools: Vec::new(),
                })
            })
            .collect()
    }
}

impl Default for McpManager {
    fn default() -> Self {
        Self::new()
    }
}

/// MCP instruction block.
#[derive(Debug, Clone)]
pub struct McpInstruction {
    pub name: String,
    pub instructions: String,
    pub tools: Vec<String>,
}
