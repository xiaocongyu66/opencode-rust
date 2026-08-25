//! Location management — resolves and manages working locations.

use std::sync::Arc;
use tokio::sync::RwLock;
use crate::schema::ids::ProjectID;
use crate::schema::location::{LocationInfo, LocationRef, LocationProject};

pub struct LocationService {
    locations: Arc<RwLock<Vec<LocationInfo>>>,
}

impl LocationService {
    pub fn new() -> Self {
        Self { locations: Arc::new(RwLock::new(vec![])) }
    }

    pub async fn resolve(&self, r#ref: &LocationRef) -> LocationInfo {
        LocationInfo {
            directory: r#ref.directory.clone(),
            workspace_id: r#ref.workspace_id.clone(),
            project: LocationProject {
                id: ProjectID::global(),
                directory: r#ref.directory.clone(),
            },
        }
    }

    pub async fn register(&self, info: LocationInfo) {
        self.locations.write().await.push(info);
    }

    pub async fn list(&self) -> Vec<LocationInfo> {
        self.locations.read().await.clone()
    }

    pub async fn find_by_directory(&self, dir: &str) -> Option<LocationInfo> {
        self.locations.read().await.iter()
            .find(|l| l.directory.as_str() == dir)
            .cloned()
    }
}

impl Default for LocationService {
    fn default() -> Self {
        Self::new()
    }
}
