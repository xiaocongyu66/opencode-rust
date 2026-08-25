//! Global application state.

use std::sync::Arc;

use crate::agent::AgentRegistry;
use crate::credential::CredentialStore;
use crate::event::EventBus;
use crate::integration::IntegrationRegistry;
use crate::model::ModelRegistry;
use crate::permission::PermissionSystem;
use crate::process::ProcessManager;
use crate::project::ProjectRegistry;
use crate::provider::ProviderRegistry;
use crate::session::store::InMemorySessionStore;
use crate::session::SessionStore;
use crate::skill::SkillRegistry;

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
