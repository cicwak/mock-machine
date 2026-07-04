use std::sync::Arc;

use crate::{
    realtime::RealtimeNotifier,
    repository::{MockRouteRepository, ObjectAssetRepository, UnknownRequestRepository},
};

#[derive(Clone)]
pub struct AppState {
    pub unknown_requests: Arc<dyn UnknownRequestRepository>,
    pub routes: Arc<dyn MockRouteRepository>,
    pub assets: Arc<dyn ObjectAssetRepository>,
    pub realtime: Arc<dyn RealtimeNotifier>,
    pub storage: &'static str,
}
