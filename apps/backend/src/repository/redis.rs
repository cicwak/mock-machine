use anyhow::Context;
use bytes::Bytes;
use chrono::Utc;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    domain::{
        ActiveMockResponse, CapturedRequest, ConvertUnknownRequest, ConvertedUnknownRequest,
        MockRoute, ObjectAsset, ResponseScenario, RouteStatus, UnknownRequest,
        UnknownRequestStatus,
    },
    repository::{
        MockRouteRepository, ObjectAssetRepository, RepositoryError, RepositoryResult,
        UnknownRequestRepository,
    },
};

#[derive(Clone)]
pub struct RedisRepository {
    client: redis::Client,
}

#[derive(Debug, Serialize, Deserialize)]
struct StoredUnknownRequest {
    request: UnknownRequest,
    body: Option<Vec<u8>>,
}

impl From<UnknownRequest> for StoredUnknownRequest {
    fn from(mut request: UnknownRequest) -> Self {
        let body = request.body.take();
        Self { request, body }
    }
}

impl From<StoredUnknownRequest> for UnknownRequest {
    fn from(mut value: StoredUnknownRequest) -> Self {
        value.request.body = value.body;
        value.request
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct StoredObjectAsset {
    content_type: Option<String>,
    body: Vec<u8>,
}

impl RedisRepository {
    pub async fn new(redis_url: &str) -> anyhow::Result<Self> {
        let client = redis::Client::open(redis_url).context("invalid REDIS_URL")?;
        let mut conn = client
            .get_multiplexed_async_connection()
            .await
            .context("failed to open Redis connection")?;
        let _: String = redis::cmd("PING")
            .query_async(&mut conn)
            .await
            .context("failed to ping Redis")?;
        Ok(Self { client })
    }

    async fn conn(&self) -> RepositoryResult<redis::aio::MultiplexedConnection> {
        self.client
            .get_multiplexed_async_connection()
            .await
            .context("failed to open Redis connection")
            .map_err(RepositoryError::Internal)
    }
}

#[async_trait::async_trait]
impl UnknownRequestRepository for RedisRepository {
    async fn capture(&self, request: CapturedRequest) -> RepositoryResult<UnknownRequest> {
        let method = request.method.to_uppercase();
        let index_key = unknown_index_key(&method, &request.path);
        let now = Utc::now();
        let mut conn = self.conn().await?;

        let id = match conn.get::<_, Option<String>>(&index_key).await? {
            Some(id) => Uuid::parse_str(&id).context("invalid unknown request id in Redis")?,
            None => {
                let id = Uuid::new_v4();
                let inserted: bool = conn.set_nx(&index_key, id.to_string()).await?;
                if inserted {
                    id
                } else {
                    let id: String = conn.get(&index_key).await?;
                    Uuid::parse_str(&id).context("invalid unknown request id in Redis")?
                }
            }
        };

        let key = unknown_key(id);
        let mut unknown = match conn.get::<_, Option<String>>(&key).await? {
            Some(json) => serde_json::from_str::<StoredUnknownRequest>(&json)
                .context("failed to decode unknown request from Redis")
                .map(UnknownRequest::from)?,
            None => UnknownRequest {
                id,
                method,
                path: request.path.clone(),
                query: request.query.clone(),
                headers: request.headers.clone(),
                body: request.body.clone(),
                first_seen_at: now,
                last_seen_at: now,
                count: 0,
                status: UnknownRequestStatus::New,
                converted_route_id: None,
            },
        };

        unknown.query = request.query;
        unknown.headers = request.headers;
        unknown.body = request.body;
        unknown.last_seen_at = now;
        unknown.count += 1;
        if unknown.status != UnknownRequestStatus::Converted {
            unknown.status = UnknownRequestStatus::New;
        }

        let json = serde_json::to_string(&StoredUnknownRequest::from(unknown.clone()))
            .context("failed to encode unknown request")?;
        let score = unknown.last_seen_at.timestamp_millis();
        let _: () = redis::pipe()
            .set(&key, json)
            .zadd(UNKNOWN_SET, unknown.id.to_string(), score)
            .query_async(&mut conn)
            .await?;

        Ok(unknown)
    }

    async fn list(
        &self,
        status: Option<UnknownRequestStatus>,
        limit: u64,
    ) -> RepositoryResult<Vec<UnknownRequest>> {
        let mut conn = self.conn().await?;
        let ids: Vec<String> = conn
            .zrevrange(UNKNOWN_SET, 0, limit.saturating_sub(1) as isize)
            .await?;
        let mut result = Vec::new();

        for id in ids {
            let id = Uuid::parse_str(&id).context("invalid unknown request id in Redis")?;
            let Some(request) = load_unknown(&mut conn, &unknown_key(id)).await? else {
                continue;
            };

            if status
                .as_ref()
                .is_none_or(|expected| request.status == *expected)
            {
                result.push(request);
            }
        }

        result.truncate(limit as usize);
        Ok(result)
    }

    async fn get(&self, id: Uuid) -> RepositoryResult<Option<UnknownRequest>> {
        let mut conn = self.conn().await?;
        load_unknown(&mut conn, &unknown_key(id)).await
    }
}

#[async_trait::async_trait]
impl MockRouteRepository for RedisRepository {
    async fn list_routes(&self) -> RepositoryResult<Vec<MockRoute>> {
        let mut conn = self.conn().await?;
        let ids: Vec<String> = conn.zrange(ROUTE_SET, 0, -1).await?;
        let mut routes = Vec::new();

        for id in ids {
            let id = Uuid::parse_str(&id).context("invalid route id in Redis")?;
            if let Some(route) = load_json::<MockRoute>(&mut conn, &route_key(id)).await? {
                routes.push(route);
            }
        }

        routes.sort_by(|left, right| left.path_pattern.cmp(&right.path_pattern));
        Ok(routes)
    }

    async fn get_route(&self, id: Uuid) -> RepositoryResult<Option<MockRoute>> {
        let mut conn = self.conn().await?;
        load_json(&mut conn, &route_key(id)).await
    }

    async fn find_active_response(
        &self,
        method: &str,
        path: &str,
    ) -> RepositoryResult<Option<ActiveMockResponse>> {
        let mut conn = self.conn().await?;
        let Some(route_id) = conn
            .get::<_, Option<String>>(route_index_key(&method.to_uppercase(), path))
            .await?
        else {
            return Ok(None);
        };

        let route_id = Uuid::parse_str(&route_id).context("invalid route id in Redis")?;
        let Some(route) = load_json::<MockRoute>(&mut conn, &route_key(route_id)).await? else {
            return Ok(None);
        };

        if route.status != RouteStatus::Active {
            return Ok(None);
        }

        let Some(scenario_id) = route.active_scenario_id else {
            return Ok(None);
        };
        let scenario = load_json::<ResponseScenario>(&mut conn, &scenario_key(scenario_id)).await?;

        Ok(scenario.map(|scenario| ActiveMockResponse { route, scenario }))
    }

    async fn convert_unknown_request(
        &self,
        id: Uuid,
        request: ConvertUnknownRequest,
    ) -> RepositoryResult<ConvertedUnknownRequest> {
        validate_convert_request(&request)?;

        let mut conn = self.conn().await?;
        let Some(mut unknown) = load_unknown(&mut conn, &unknown_key(id)).await? else {
            return Err(RepositoryError::NotFound);
        };

        if unknown.status == UnknownRequestStatus::Converted {
            return Err(RepositoryError::Conflict(
                "unknown request is already converted".to_string(),
            ));
        }

        if unknown.path == "/mockadmin"
            || unknown.path.starts_with("/mockadmin/")
            || unknown.path == "/mockadminapi"
            || unknown.path.starts_with("/mockadminapi/")
        {
            return Err(RepositoryError::Validation(
                "admin paths cannot be converted to mock routes".to_string(),
            ));
        }

        let route_index = route_index_key(&unknown.method, &unknown.path);
        if conn.exists::<_, bool>(&route_index).await? {
            return Err(RepositoryError::Conflict(
                "route already exists for this method and path".to_string(),
            ));
        }

        let now = Utc::now();
        let route_id = Uuid::new_v4();
        let scenario_id = Uuid::new_v4();
        let route = MockRoute {
            id: route_id,
            method: unknown.method.clone(),
            path_pattern: unknown.path.clone(),
            name: request.name.unwrap_or_else(|| {
                format!(
                    "{} {}",
                    unknown.method,
                    unknown.path.trim_start_matches('/').replace('/', " / ")
                )
            }),
            tags: request.tags,
            status: RouteStatus::Active,
            active_scenario_id: Some(scenario_id),
            created_at: now,
            updated_at: now,
        };
        let scenario = ResponseScenario {
            id: scenario_id,
            route_id,
            name: request.scenario.name,
            kind: request.scenario.kind,
            status_code: request.scenario.status_code,
            response_headers: request.scenario.response_headers,
            response_body: request.scenario.response_body,
            delay_ms: request.scenario.delay_ms,
            selection_rules: request.scenario.selection_rules,
            created_at: now,
            updated_at: now,
        };

        unknown.status = UnknownRequestStatus::Converted;
        unknown.converted_route_id = Some(route_id);

        let _: () = redis::pipe()
            .set(route_key(route_id), serde_json::to_string(&route)?)
            .set(scenario_key(scenario_id), serde_json::to_string(&scenario)?)
            .set(route_index, route_id.to_string())
            .zadd(
                ROUTE_SET,
                route_id.to_string(),
                route.created_at.timestamp_millis(),
            )
            .set(
                unknown_key(id),
                serde_json::to_string(&StoredUnknownRequest::from(unknown.clone()))?,
            )
            .query_async(&mut conn)
            .await?;

        Ok(ConvertedUnknownRequest {
            route,
            scenario,
            unknown_request: unknown,
        })
    }
}

#[async_trait::async_trait]
impl ObjectAssetRepository for RedisRepository {
    async fn put(
        &self,
        object_key: &str,
        content_type: Option<&str>,
        body: Bytes,
    ) -> RepositoryResult<ObjectAsset> {
        let mut conn = self.conn().await?;
        let asset = StoredObjectAsset {
            content_type: content_type.map(ToOwned::to_owned),
            body: body.to_vec(),
        };

        let _: () = conn
            .set(object_key_key(object_key), serde_json::to_string(&asset)?)
            .await?;

        Ok(ObjectAsset {
            bucket: "redis".to_string(),
            object_key: object_key.to_string(),
            content_type: asset.content_type,
            size_bytes: body.len() as i64,
        })
    }

    async fn get(&self, object_key: &str) -> RepositoryResult<Option<Bytes>> {
        let mut conn = self.conn().await?;
        let Some(asset) =
            load_json::<StoredObjectAsset>(&mut conn, &object_key_key(object_key)).await?
        else {
            return Ok(None);
        };

        Ok(Some(Bytes::from(asset.body)))
    }
}

async fn load_json<T>(
    conn: &mut redis::aio::MultiplexedConnection,
    key: &str,
) -> RepositoryResult<Option<T>>
where
    T: for<'de> Deserialize<'de>,
{
    let Some(json) = conn.get::<_, Option<String>>(key).await? else {
        return Ok(None);
    };

    serde_json::from_str(&json)
        .context("failed to decode JSON from Redis")
        .map(Some)
        .map_err(RepositoryError::Internal)
}

async fn load_unknown(
    conn: &mut redis::aio::MultiplexedConnection,
    key: &str,
) -> RepositoryResult<Option<UnknownRequest>> {
    load_json::<StoredUnknownRequest>(conn, key)
        .await
        .map(|value| value.map(UnknownRequest::from))
}

fn validate_convert_request(request: &ConvertUnknownRequest) -> RepositoryResult<()> {
    if !(100..=599).contains(&request.scenario.status_code) {
        return Err(RepositoryError::Validation(
            "scenario.status_code must be between 100 and 599".to_string(),
        ));
    }

    if request.scenario.delay_ms < 0 {
        return Err(RepositoryError::Validation(
            "scenario.delay_ms must be non-negative".to_string(),
        ));
    }

    if !request.scenario.response_headers.is_object() {
        return Err(RepositoryError::Validation(
            "scenario.response_headers must be a JSON object".to_string(),
        ));
    }

    if !request.scenario.selection_rules.is_object() {
        return Err(RepositoryError::Validation(
            "scenario.selection_rules must be a JSON object".to_string(),
        ));
    }

    Ok(())
}

const UNKNOWN_SET: &str = "mock-machine:unknown";
const ROUTE_SET: &str = "mock-machine:routes";

fn unknown_key(id: Uuid) -> String {
    format!("mock-machine:unknown:{id}")
}

fn unknown_index_key(method: &str, path: &str) -> String {
    format!("mock-machine:unknown-index:{method}:{path}")
}

fn route_key(id: Uuid) -> String {
    format!("mock-machine:route:{id}")
}

fn route_index_key(method: &str, path: &str) -> String {
    format!("mock-machine:route-index:{method}:{path}")
}

fn scenario_key(id: Uuid) -> String {
    format!("mock-machine:scenario:{id}")
}

fn object_key_key(object_key: &str) -> String {
    format!("mock-machine:object:{object_key}")
}
