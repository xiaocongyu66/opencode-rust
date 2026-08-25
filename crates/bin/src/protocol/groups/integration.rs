//! Integration routes — `packages/protocol/src/groups/integration.ts`
//!
//! Integration routes cover discovery, key-based and OAuth-based
//! authentication, and attempt lifecycle management.

use axum::Router;

use crate::protocol::api::ApiGroup;

// ---------------------------------------------------------------------------
// Route paths
// ---------------------------------------------------------------------------

/// `GET` — List available integrations and their authentication methods.
pub const INTEGRATION_LIST: &str = "/api/integration";
/// `GET` — Get one integration and its authentication methods.
pub const INTEGRATION_GET: &str = "/api/integration/:integrationID";
/// `POST` — Run a key authentication method and store the credential.
pub const INTEGRATION_CONNECT_KEY: &str = "/api/integration/:integrationID/connect/key";
/// `POST` — Begin an OAuth attempt and return authorization details.
pub const INTEGRATION_CONNECT_OAUTH: &str = "/api/integration/:integrationID/connect/oauth";
/// `GET` — Poll the current status of an OAuth attempt.
pub const INTEGRATION_ATTEMPT_STATUS: &str = "/api/integration/attempt/:attemptID";
/// `POST` — Complete a code-based OAuth attempt and store the credential.
pub const INTEGRATION_ATTEMPT_COMPLETE: &str = "/api/integration/attempt/:attemptID/complete";
/// `DELETE` — Cancel an OAuth attempt and release its resources.
pub const INTEGRATION_ATTEMPT_CANCEL: &str = "/api/integration/attempt/:attemptID";

/// Integration API group.
pub struct IntegrationGroup;

impl ApiGroup for IntegrationGroup {
    type State = ();
    fn routes() -> Router<()> {
        Router::new()
    }
}
