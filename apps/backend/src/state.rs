use std::sync::Arc;

use tokio::sync::RwLock;
use uuid::Uuid;

use crate::{
    realtime::RealtimeNotifier,
    repository::{
        MockRouteRepository, ObjectAssetRepository, ProjectRepository, UnknownRequestRepository,
    },
};

#[derive(Clone)]
pub struct AppState {
    pub projects: Arc<dyn ProjectRepository>,
    pub unknown_requests: Arc<dyn UnknownRequestRepository>,
    pub routes: Arc<dyn MockRouteRepository>,
    pub assets: Arc<dyn ObjectAssetRepository>,
    pub realtime: Arc<dyn RealtimeNotifier>,
    pub active_project_id: Arc<RwLock<Option<Uuid>>>,
    pub storage: &'static str,
}
