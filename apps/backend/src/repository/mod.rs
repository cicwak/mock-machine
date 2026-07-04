use async_trait::async_trait;
use bytes::Bytes;
use uuid::Uuid;

use crate::domain::{
    ActiveMockResponse, CapturedRequest, ConvertUnknownRequest, ConvertedUnknownRequest, MockRoute,
    ObjectAsset, UnknownRequest, UnknownRequestStatus,
};

pub mod in_memory;
pub mod minio;
pub mod postgres;
pub mod redis;

#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("resource not found")]
    NotFound,
    #[error("resource conflict: {0}")]
    Conflict(String),
    #[error("validation failed: {0}")]
    Validation(String),
    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

impl From<::redis::RedisError> for RepositoryError {
    fn from(value: ::redis::RedisError) -> Self {
        Self::Internal(value.into())
    }
}

impl From<serde_json::Error> for RepositoryError {
    fn from(value: serde_json::Error) -> Self {
        Self::Internal(value.into())
    }
}

pub type RepositoryResult<T> = Result<T, RepositoryError>;

#[async_trait]
pub trait UnknownRequestRepository: Send + Sync {
    async fn capture(&self, request: CapturedRequest) -> RepositoryResult<UnknownRequest>;

    async fn list(
        &self,
        status: Option<UnknownRequestStatus>,
        limit: u64,
    ) -> RepositoryResult<Vec<UnknownRequest>>;

    async fn get(&self, id: Uuid) -> RepositoryResult<Option<UnknownRequest>>;
}

#[async_trait]
pub trait MockRouteRepository: Send + Sync {
    async fn list_routes(&self) -> RepositoryResult<Vec<MockRoute>>;

    async fn get_route(&self, id: Uuid) -> RepositoryResult<Option<MockRoute>>;

    async fn find_active_response(
        &self,
        method: &str,
        path: &str,
    ) -> RepositoryResult<Option<ActiveMockResponse>>;

    async fn convert_unknown_request(
        &self,
        id: Uuid,
        request: ConvertUnknownRequest,
    ) -> RepositoryResult<ConvertedUnknownRequest>;
}

#[async_trait]
pub trait ObjectAssetRepository: Send + Sync {
    async fn put(
        &self,
        object_key: &str,
        content_type: Option<&str>,
        body: Bytes,
    ) -> RepositoryResult<ObjectAsset>;

    async fn get(&self, object_key: &str) -> RepositoryResult<Option<Bytes>>;
}
