//! Global application state.

use std::sync::Arc;

use crate::core::agent::AgentRegistry;
use crate::core::credential::CredentialStore;
use crate::core::event::EventBus;
use crate::core::integration::IntegrationRegistry;
use crate::core::model::ModelRegistry;
use crate::core::permission::PermissionSystem;
use crate::core::process::ProcessManager;
use crate::core::project::ProjectRegistry;
use crate::core::provider::ProviderRegistry;
use crate::core::session::store::InMemorySessionStore;
use crate::core::session::SessionStore;
use crate::core::skill::SkillRegistry;

pub struct AppState {
    pub agents: AgentRegistry,
    pub models: ModelRegistry,
    pub providers: ProviderRegistry,
    pub projects: ProjectRegistry,
    pub credentials: CredentialStore,
    pub permissions: PermissionSystem,
    pub integrations: IntegrationRegistry,
    pub skills: SkillRegistry,
    pub events: EventBus,
    pub processes: ProcessManager,
    pub sessions: Arc<dyn SessionStore>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            agents: AgentRegistry::new(),
            models: ModelRegistry::new(),
            providers: ProviderRegistry::new(),
            projects: ProjectRegistry::new(),
            credentials: CredentialStore::new(),
            permissions: PermissionSystem::new(),
            integrations: IntegrationRegistry::new(),
            skills: SkillRegistry::new(),
            events: EventBus::new(1024),
            processes: ProcessManager::new(),
            sessions: Arc::new(InMemorySessionStore::new()),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
