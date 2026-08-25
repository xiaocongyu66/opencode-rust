//! API route group trait for composing axum routers.
//!
//! Each protocol group implements [`ApiGroup`] to expose its routes as an
//! axum [`Router`]. The top-level API is assembled by merging all group
//! routers and applying global middleware (authorization + schema error).

use axum::Router;

/// A group of related API routes that can be merged into a top-level router.
pub trait ApiGroup {
    type State;
    fn routes() -> Router<Self::State>;
}
