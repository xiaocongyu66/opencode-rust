//! Credential routes — `packages/protocol/src/groups/credential.ts`
//!
//! Credential routes manage stored integration credentials (update label,
//! remove).

use axum::Router;

use crate::protocol::api::ApiGroup;

/// `PATCH` — Update a stored credential's label.
pub const CREDENTIAL_UPDATE: &str = "/api/credential/:credentialID";
/// `DELETE` — Remove a stored integration credential.
pub const CREDENTIAL_REMOVE: &str = "/api/credential/:credentialID";

/// Credential API group.
pub struct CredentialGroup;

impl ApiGroup for CredentialGroup {
    type State = ();
    fn routes() -> Router<()> {
        Router::new()
    }
}
