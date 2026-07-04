use anyhow::Context;
use redis::AsyncCommands;
use uuid::Uuid;

use crate::{
    domain::{
        ActiveMockResponse, ConvertUnknownRequest, ConvertedUnknownRequest, CreateScenario,
        MockRoute, ResponseScenario, UpsertRoute,
    },
    repository::{
        MockRouteRepository, RepositoryError, RepositoryResult, postgres::PostgresRepository,
    },
};

#[derive(Clone)]
pub struct CachedRouteRepository {
    primary: PostgresRepository,
    redis: redis::Client,
    ttl_seconds: u64,
}

impl CachedRouteRepository {
    pub async fn new(
        primary: PostgresRepository,
        redis_url: &str,
        ttl_seconds: u64,
    ) -> anyhow::Result<Self> {
        let redis = redis::Client::open(redis_url).context("invalid REDIS_URL")?;
        let mut conn = redis
            .get_multiplexed_async_connection()
            .await
            .context("failed to open Redis connection")?;
        let _: String = redis::cmd("PING")
            .query_async(&mut conn)
            .await
            .context("failed to ping Redis")?;

        Ok(Self {
            primary,
            redis,
            ttl_seconds,
        })
    }

    async fn conn(&self) -> RepositoryResult<redis::aio::MultiplexedConnection> {
        self.redis
            .get_multiplexed_async_connection()
            .await
            .context("failed to open Redis connection")
            .map_err(RepositoryError::Internal)
    }

    async fn invalidate_route(&self, route: &MockRoute) -> RepositoryResult<()> {
        let mut conn = self.conn().await?;
        let _: () = conn
            .del(active_response_key(
                route.project_id,
                &route.method,
                &route.path_pattern,
            ))
            .await?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl MockRouteRepository for CachedRouteRepository {
    async fn list_routes(&self, project_id: Uuid) -> RepositoryResult<Vec<MockRoute>> {
        self.primary.list_routes(project_id).await
    }

    async fn get_route(&self, id: Uuid) -> RepositoryResult<Option<MockRoute>> {
        self.primary.get_route(id).await
    }

    async fn upsert_route(
        &self,
        project_id: Uuid,
        id: Option<Uuid>,
        request: UpsertRoute,
    ) -> RepositoryResult<MockRoute> {
        let old_route = match id {
            Some(id) => self.primary.get_route(id).await?,
            None => None,
        };
        if let Some(route) = &old_route {
            self.invalidate_route(route).await?;
        }
        let route = self.primary.upsert_route(project_id, id, request).await?;
        self.invalidate_route(&route).await?;
        Ok(route)
    }

    async fn list_profiles(&self, route_id: Uuid) -> RepositoryResult<Vec<ResponseScenario>> {
        self.primary.list_profiles(route_id).await
    }

    async fn upsert_profile(
        &self,
        route_id: Uuid,
        profile_id: Option<Uuid>,
        request: CreateScenario,
    ) -> RepositoryResult<ResponseScenario> {
        let route = self.primary.get_route(route_id).await?;
        if let Some(route) = &route {
            self.invalidate_route(route).await?;
        }
        let profile = self
            .primary
            .upsert_profile(route_id, profile_id, request)
            .await?;
        if let Some(route) = &route {
            self.invalidate_route(route).await?;
        }
        Ok(profile)
    }

    async fn set_active_profile(
        &self,
        route_id: Uuid,
        profile_id: Uuid,
    ) -> RepositoryResult<MockRoute> {
        let route = self.primary.get_route(route_id).await?;
        if let Some(route) = &route {
            self.invalidate_route(route).await?;
        }
        let route = self
            .primary
            .set_active_profile(route_id, profile_id)
            .await?;
        self.invalidate_route(&route).await?;
        Ok(route)
    }

    async fn find_active_response(
        &self,
        project_id: Uuid,
        method: &str,
        path: &str,
    ) -> RepositoryResult<Option<ActiveMockResponse>> {
        let key = active_response_key(project_id, &method.to_uppercase(), path);
        let mut conn = self.conn().await?;
        if let Some(json) = conn.get::<_, Option<String>>(&key).await? {
            let cached =
                serde_json::from_str(&json).context("failed to decode cached active response")?;
            return Ok(Some(cached));
        }

        let response = self
            .primary
            .find_active_response(project_id, method, path)
            .await?;
        if let Some(response) = &response {
            let _: () = conn
                .set_ex(&key, serde_json::to_string(response)?, self.ttl_seconds)
                .await?;
        }
        Ok(response)
    }

    async fn convert_unknown_request(
        &self,
        project_id: Uuid,
        id: Uuid,
        request: ConvertUnknownRequest,
    ) -> RepositoryResult<ConvertedUnknownRequest> {
        let converted = self
            .primary
            .convert_unknown_request(project_id, id, request)
            .await?;
        self.invalidate_route(&converted.route).await?;
        Ok(converted)
    }
}

fn active_response_key(project_id: Uuid, method: &str, path: &str) -> String {
    format!(
        "mock-machine:route-cache:active:{project_id}:{}:{}",
        method.to_uppercase(),
        path
    )
}
