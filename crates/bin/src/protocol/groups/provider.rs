//! Provider routes — `packages/protocol/src/groups/provider.ts`

use axum::Router;

use crate::protocol::api::ApiGroup;

/// `GET` — List active AI providers.
pub const PROVIDER_LIST: &str = "/api/provider";
/// `GET` — Get a single provider by ID.
pub const PROVIDER_GET: &str = "/api/provider/:providerID";

/// Provider API group.
pub struct ProviderGroup;

impl ApiGroup for ProviderGroup {
    type State = ();
    fn routes() -> Router<()> {
        Router::new()
    }
}
