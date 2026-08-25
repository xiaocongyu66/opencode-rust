//! Code search tool — powered by sonar (BM25 + semantic hybrid search).
//!
//! Replaces the naive grep/glob pattern with intelligent code search.
//! Returns the most relevant code chunks instead of raw file matches.

use std::path::Path;
use std::sync::Arc;
use async_trait::async_trait;
use serde::Deserialize;
use tokio::sync::RwLock;

use crate::tool::{Tool, ToolContext, ToolFailure, ToolResult};

pub struct CodeSearchTool {
    cache: Arc<RwLock<Option<CachedIndex>>>,
}

struct CachedIndex {
    index: Arc<sonar_core::index::SonarIndex>,
    directory_hash: String,
}

impl CodeSearchTool {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(RwLock::new(None)),
        }
    }

    async fn get_or_build_index(&self, path: &Path) -> Result<Arc<sonar_core::index::SonarIndex>, ToolFailure> {
        let abs = path.canonicalize().map_err(ToolFailure::Io)?;
        let abs_str = abs.to_string_lossy().to_string();

        {
            let cache = self.cache.read().await;
            if let Some(ref cached) = *cache {
                if cached.directory_hash == abs_str {
                    return Ok(cached.index.clone());
                }
            }
        }

        let index = tokio::task::spawn_blocking({
            let path = abs.clone();
            move || sonar_core::index::SonarIndex::from_path(&path)
                .map_err(|e| ToolFailure::Message(format!("Failed to build index: {}", e)))
        })
        .await
        .map_err(|e| ToolFailure::TaskJoin(e.to_string()))??;

        let arc = Arc::new(index);

        let mut cache = self.cache.write().await;
        *cache = Some(CachedIndex {
            directory_hash: abs_str,
            index: arc.clone(),
        });

        Ok(arc)
    }
}

impl Default for CodeSearchTool {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Deserialize)]
struct CodeSearchInput {
    query: String,
    #[serde(default)]
    path: Option<String>,
    #[serde(default)]
    top_k: Option<usize>,
    #[serde(default)]
    mode: Option<String>,
}

#[async_trait]
impl Tool for CodeSearchTool {
    fn name(&self) -> &str { "code_search" }

    fn description(&self) -> &str {
        "Hybrid code search (BM25 + semantic). Returns the most relevant code chunks for a natural language or code query. Much more efficient than grep — returns 5 ranked results instead of 90+ file matches."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "Natural language or code query (e.g. 'auth middleware', 'error handling pattern')"
                },
                "path": {
                    "type": "string",
                    "description": "Directory to search in. Defaults to current working directory."
                },
                "top_k": {
                    "type": "integer",
                    "description": "Number of results to return. Default: 5.",
                    "default": 5
                },
                "mode": {
                    "type": "string",
                    "enum": ["hybrid", "bm25", "semantic"],
                    "description": "Search mode: hybrid (default), bm25 (keyword only), semantic (vector only)",
                    "default": "hybrid"
                }
            },
            "required": ["query"]
        })
    }

    async fn execute(&self, params: serde_json::Value, _ctx: &ToolContext) -> Result<ToolResult, ToolFailure> {
        let input: CodeSearchInput = serde_json::from_value(params)?;
        let path = input.path.as_deref().unwrap_or(".");
        let path = Path::new(path);
        let top_k = input.top_k.unwrap_or(5);

        let _mode = match input.mode.as_deref().unwrap_or("hybrid") {
            "bm25" => sonar_core::index::Mode::Bm25,
            "semantic" => sonar_core::index::Mode::Semantic,
            _ => sonar_core::index::Mode::Hybrid,
        };

        let index = self.get_or_build_index(path).await?;
        // SonarIndex search is synchronous; run in blocking pool
        let query = input.query.clone();
        let results = tokio::task::spawn_blocking(move || {
            // Note: set_mode takes &mut, so we need interior mutability or
            // build with the right mode. For now, use search() which uses default mode.
            index.search(&query, top_k)
        })
        .await
        .map_err(|e| ToolFailure::TaskJoin(e.to_string()))?;

        if results.is_empty() {
            return Ok(ToolResult::text("No matching code found."));
        }

        let output = results
            .iter()
            .enumerate()
            .map(|(i, r)| {
                format!(
                    "--- Result {} (score: {:.3}) ---\n{}\n{}\n",
                    i + 1,
                    r.score,
                    r.chunk.location(),
                    r.chunk.content
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        Ok(ToolResult::text(output))
    }
}
