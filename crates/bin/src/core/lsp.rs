//! LSP (Language Server Protocol) integration.
//!
//! Ported from `lsp/lsp.ts`.
//! Manages language server connections, diagnostics, and symbol queries.

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;

/// LSP position.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Position {
    pub line: u64,
    pub character: u64,
}

/// LSP range.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Range {
    pub start: Position,
    pub end: Position,
}

/// LSP symbol kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum SymbolKind {
    File = 1,
    Module = 2,
    Namespace = 3,
    Package = 4,
    Class = 5,
    Method = 6,
    Property = 7,
    Field = 8,
    Constructor = 9,
    Enum = 10,
    Interface = 11,
    Function = 12,
    Variable = 13,
    Constant = 14,
    String = 15,
    Number = 16,
    Boolean = 17,
    Array = 18,
    Object = 19,
    Key = 20,
    Null = 21,
    EnumMember = 22,
    Struct = 23,
    Event = 24,
    Operator = 25,
    TypeParameter = 26,
}

/// Symbol kinds to include in queries.
pub const INCLUDED_SYMBOL_KINDS: &[SymbolKind] = &[
    SymbolKind::Class,
    SymbolKind::Function,
    SymbolKind::Method,
    SymbolKind::Interface,
    SymbolKind::Variable,
    SymbolKind::Constant,
    SymbolKind::Struct,
    SymbolKind::Enum,
];

/// LSP document symbol.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DocumentSymbol {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    pub kind: u32,
    pub range: Range,
    pub selection_range: Range,
}

/// LSP workspace symbol.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WorkspaceSymbol {
    pub name: String,
    pub kind: u32,
    pub location: SymbolLocation,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SymbolLocation {
    pub uri: String,
    pub range: Range,
}

/// LSP server status.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LspServerStatus {
    pub id: String,
    pub name: String,
    pub root: String,
    pub status: LspConnectionStatus,
}

/// LSP connection status.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LspConnectionStatus {
    Connected,
    Error,
}

/// LSP diagnostic.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Diagnostic {
    pub range: Range,
    pub severity: DiagnosticSeverity,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

/// Diagnostic severity.
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Information,
    Hint,
}

/// LSP server configuration.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LspServerInfo {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    pub languages: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Vec<String>>,
}

/// LSP manager — coordinates language server processes.
pub struct LspManager {
    servers: Arc<RwLock<HashMap<String, LspServerStatus>>>,
}

impl LspManager {
    pub fn new() -> Self {
        Self {
            servers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn list_servers(&self) -> Vec<LspServerStatus> {
        self.servers.read().await.values().cloned().collect()
    }

    pub async fn get_server(&self, id: &str) -> Option<LspServerStatus> {
        self.servers.read().await.get(id).cloned()
    }

    pub async fn add_server(&self, server: LspServerStatus) {
        self.servers.write().await.insert(server.id.clone(), server);
    }

    pub async fn remove_server(&self, id: &str) {
        self.servers.write().await.remove(id);
    }

    pub async fn document_symbols(&self, _uri: &str) -> anyhow::Result<Vec<DocumentSymbol>> {
        Ok(Vec::new())
    }

    pub async fn workspace_symbols(&self, _query: &str) -> anyhow::Result<Vec<WorkspaceSymbol>> {
        Ok(Vec::new())
    }

    pub async fn diagnostics(&self, _uri: &str) -> anyhow::Result<Vec<Diagnostic>> {
        Ok(Vec::new())
    }
}

impl Default for LspManager {
    fn default() -> Self {
        Self::new()
    }
}
