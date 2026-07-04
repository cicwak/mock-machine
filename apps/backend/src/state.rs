use std::sync::Arc;

use crate::repository::{MockRouteRepository, ObjectAssetRepository, UnknownRequestRepository};

#[derive(Clone)]
pub struct AppState {
    pub unknown_requests: Arc<dyn UnknownRequestRepository>,
    pub routes: Arc<dyn MockRouteRepository>,
    pub assets: Arc<dyn ObjectAssetRepository>,
    pub storage: &'static str,
}
