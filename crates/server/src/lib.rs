//! opencode-server crate
//!
//! HTTP server exposing the opencode API over a local socket.

pub mod routes;
pub mod handlers;

use std::sync::Arc;
use axum::Router;
use opencode_core::state::AppState;

pub struct Server {
    state: Arc<AppState>,
}

impl Server {
    pub fn new() -> Self {
        Self { state: Arc::new(AppState::new()) }
    }

    pub fn state(&self) -> &Arc<AppState> {
        &self.state
    }

    pub fn router(&self) -> Router {
        routes::build_router(self.state.clone())
    }

    pub async fn serve(&self, addr: &str) -> Result<(), std::io::Error> {
        let app = self.router();
        let listener = tokio::net::TcpListener::bind(addr).await?;
        tracing::info!("Server listening on {}", addr);
        axum::serve(listener, app).await?;
        Ok(())
    }
}

impl Default for Server {
    fn default() -> Self {
        Self::new()
    }
}
