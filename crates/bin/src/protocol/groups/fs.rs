//! Filesystem routes — `packages/protocol/src/groups/fs.ts`
//!
//! All filesystem routes are location-scoped and operate relative to the
//! requested location's directory.

use axum::Router;

use crate::protocol::api::ApiGroup;

/// `GET` — Serve one file relative to the requested location.
pub const FS_READ: &str = "/api/fs/read/*";
/// `GET` — List direct children of a directory relative to the location.
pub const FS_LIST: &str = "/api/fs/list";
/// `GET` — Find recursively ranked filesystem entries.
pub const FS_FIND: &str = "/api/fs/find";

/// Filesystem API group.
pub struct FileSystemGroup;

impl ApiGroup for FileSystemGroup {
    type State = ();
    fn routes() -> Router<()> {
        Router::new()
    }
}
