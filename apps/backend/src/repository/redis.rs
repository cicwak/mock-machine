use anyhow::Context;
use bytes::Bytes;
use chrono::Utc;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    domain::{
        ActiveMockResponse, CapturedRequest, ConvertUnknownRequest, ConvertedUnknownRequest,
        CreateProject, CreateScenario, MockRoute, ObjectAsset, ProfileKind, Project,
        ResponseScenario, RouteStatus, UnknownRequest, UnknownRequestStatus, UpsertRoute,
        is_valid_http_method,
    },
    repository::{
        MockRouteRepository, ObjectAssetRepository, ProjectRepository, RepositoryError,
        RepositoryResult, UnknownRequestRepository,
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
impl ProjectRepository for RedisRepository {
    async fn list_projects(&self) -> RepositoryResult<Vec<Project>> {
        let mut conn = self.conn().await?;
        ensure_default_project(&mut conn).await?;
        let ids: Vec<String> = conn.zrange(PROJECT_SET, 0, -1).await?;
        let mut projects = Vec::new();

        for id in ids {
            let id = Uuid::parse_str(&id).context("invalid project id in Redis")?;
            if let Some(project) = load_json::<Project>(&mut conn, &project_key(id)).await? {
                projects.push(ensure_project_key(&mut conn, project).await?);
            }
        }

        projects.sort_by(|left, right| left.name.cmp(&right.name));
        Ok(projects)
    }

    async fn get_project(&self, id: Uuid) -> RepositoryResult<Option<Project>> {
        let mut conn = self.conn().await?;
        ensure_default_project(&mut conn).await?;
        match load_json(&mut conn, &project_key(id)).await? {
            Some(project) => Ok(Some(ensure_project_key(&mut conn, project).await?)),
            None => Ok(None),
        }
    }

    async fn get_project_by_key(&self, key: &str) -> RepositoryResult<Option<Project>> {
        let mut conn = self.conn().await?;
        ensure_default_project(&mut conn).await?;
        let key = normalize_project_key(key)?;
        let Some(id) = conn
            .get::<_, Option<String>>(project_key_index_key(&key))
            .await?
        else {
            return Ok(None);
        };
        let id = Uuid::parse_str(&id).context("invalid project id in Redis")?;
        match load_json(&mut conn, &project_key(id)).await? {
            Some(project) => Ok(Some(ensure_project_key(&mut conn, project).await?)),
            None => Ok(None),
        }
    }

    async fn create_project(&self, request: CreateProject) -> RepositoryResult<Project> {
        validate_project_request(&request)?;
        let mut conn = self.conn().await?;
        ensure_default_project(&mut conn).await?;
        let name = request.name.trim().to_string();
        let name_index = project_name_index_key(&name);
        if conn.exists::<_, bool>(&name_index).await? {
            return Err(RepositoryError::Conflict(
                "project already exists with this name".to_string(),
            ));
        }

        let now = Utc::now();
        let key = generate_project_key(&mut conn).await?;
        let project = Project {
            id: Uuid::new_v4(),
            name,
            key,
            created_at: now,
            updated_at: now,
        };
        let _: () = redis::pipe()
            .set(project_key(project.id), serde_json::to_string(&project)?)
            .set(name_index, project.id.to_string())
            .set(project_key_index_key(&project.key), project.id.to_string())
            .zadd(
                PROJECT_SET,
                project.id.to_string(),
                project.created_at.timestamp_millis(),
            )
            .query_async(&mut conn)
            .await?;
        Ok(project)
    }

    async fn rotate_project_key(&self, id: Uuid) -> RepositoryResult<Project> {
        let mut conn = self.conn().await?;
        ensure_default_project(&mut conn).await?;
        let Some(mut project) = load_json::<Project>(&mut conn, &project_key(id)).await? else {
            return Err(RepositoryError::NotFound);
        };

        project = ensure_project_key(&mut conn, project).await?;
        let old_key = project.key.clone();
        project.key = generate_project_key(&mut conn).await?;
        project.updated_at = Utc::now();

        let _: () = redis::pipe()
            .set(project_key(project.id), serde_json::to_string(&project)?)
            .del(project_key_index_key(&old_key))
            .set(project_key_index_key(&project.key), project.id.to_string())
            .query_async(&mut conn)
            .await?;

        Ok(project)
    }
}

#[async_trait::async_trait]
impl UnknownRequestRepository for RedisRepository {
    async fn capture(
        &self,
        project_id: Uuid,
        request: CapturedRequest,
    ) -> RepositoryResult<UnknownRequest> {
        let method = request.method.to_uppercase();
        let index_key = unknown_index_key(project_id, &method, &request.path);
        let now = Utc::now();
        let mut conn = self.conn().await?;
        ensure_project_exists(&mut conn, project_id).await?;

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
                project_id,
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
            .zadd(unknown_set_key(project_id), unknown.id.to_string(), score)
            .query_async(&mut conn)
            .await?;

        Ok(unknown)
    }

    async fn list(
        &self,
        project_id: Uuid,
        status: Option<UnknownRequestStatus>,
        limit: u64,
    ) -> RepositoryResult<Vec<UnknownRequest>> {
        let mut conn = self.conn().await?;
        let ids: Vec<String> = conn
            .zrevrange(
                unknown_set_key(project_id),
                0,
                limit.saturating_sub(1) as isize,
            )
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
    async fn list_routes(&self, project_id: Uuid) -> RepositoryResult<Vec<MockRoute>> {
        let mut conn = self.conn().await?;
        let ids: Vec<String> = conn.zrange(route_set_key(project_id), 0, -1).await?;
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

    async fn upsert_route(
        &self,
        project_id: Uuid,
        id: Option<Uuid>,
        request: UpsertRoute,
    ) -> RepositoryResult<MockRoute> {
        validate_route_request(&request)?;

        let mut conn = self.conn().await?;
        ensure_project_exists(&mut conn, project_id).await?;
        let id = id.unwrap_or_else(Uuid::new_v4);
        let now = Utc::now();
        let created_at = load_json::<MockRoute>(&mut conn, &route_key(id))
            .await?
            .map(|route| route.created_at)
            .unwrap_or(now);
        let route = MockRoute {
            id,
            project_id,
            method: request.method.to_uppercase(),
            path_pattern: request.path_pattern,
            name: request.name,
            tags: request.tags,
            status: request.status,
            active_scenario_id: request.active_scenario_id,
            created_at,
            updated_at: now,
        };
        let _: () = redis::pipe()
            .set(route_key(id), serde_json::to_string(&route)?)
            .set(
                route_index_key(project_id, &route.method, &route.path_pattern),
                id.to_string(),
            )
            .zadd(
                route_set_key(project_id),
                id.to_string(),
                created_at.timestamp_millis(),
            )
            .query_async(&mut conn)
            .await?;
        Ok(route)
    }

    async fn list_profiles(&self, route_id: Uuid) -> RepositoryResult<Vec<ResponseScenario>> {
        let mut conn = self.conn().await?;
        if load_json::<MockRoute>(&mut conn, &route_key(route_id))
            .await?
            .is_none()
        {
            return Err(RepositoryError::NotFound);
        }
        let ids: Vec<String> = conn.zrange(route_profiles_key(route_id), 0, -1).await?;
        let mut profiles = Vec::new();
        for id in ids {
            let id = Uuid::parse_str(&id).context("invalid profile id in Redis")?;
            if let Some(profile) = load_json(&mut conn, &scenario_key(id)).await? {
                profiles.push(profile);
            }
        }
        profiles.sort_by(|left: &ResponseScenario, right| left.name.cmp(&right.name));
        Ok(profiles)
    }

    async fn upsert_profile(
        &self,
        route_id: Uuid,
        profile_id: Option<Uuid>,
        request: CreateScenario,
    ) -> RepositoryResult<ResponseScenario> {
        validate_profile_request(&request)?;
        let mut conn = self.conn().await?;
        if load_json::<MockRoute>(&mut conn, &route_key(route_id))
            .await?
            .is_none()
        {
            return Err(RepositoryError::NotFound);
        }
        let id = profile_id.unwrap_or_else(Uuid::new_v4);
        let now = Utc::now();
        let created_at = load_json::<ResponseScenario>(&mut conn, &scenario_key(id))
            .await?
            .map(|profile| profile.created_at)
            .unwrap_or(now);
        let profile = ResponseScenario {
            id,
            route_id,
            name: request.name,
            profile_kind: request.profile_kind,
            kind: request.kind,
            proxy_url: request.proxy_url,
            status_code: request.status_code,
            response_headers: request.response_headers,
            response_body: request.response_body,
            delay_ms: request.delay_ms,
            selection_rules: request.selection_rules,
            created_at,
            updated_at: now,
        };
        let _: () = redis::pipe()
            .set(scenario_key(id), serde_json::to_string(&profile)?)
            .zadd(
                route_profiles_key(route_id),
                id.to_string(),
                created_at.timestamp_millis(),
            )
            .query_async(&mut conn)
            .await?;
        Ok(profile)
    }

    async fn set_active_profile(
        &self,
        route_id: Uuid,
        profile_id: Uuid,
    ) -> RepositoryResult<MockRoute> {
        let mut conn = self.conn().await?;
        let Some(mut route) = load_json::<MockRoute>(&mut conn, &route_key(route_id)).await? else {
            return Err(RepositoryError::NotFound);
        };
        let profile = load_json::<ResponseScenario>(&mut conn, &scenario_key(profile_id)).await?;
        if profile.is_none_or(|profile| profile.route_id != route_id) {
            return Err(RepositoryError::NotFound);
        }
        route.active_scenario_id = Some(profile_id);
        route.updated_at = Utc::now();
        let _: () = conn
            .set(route_key(route_id), serde_json::to_string(&route)?)
            .await?;
        Ok(route)
    }

    async fn find_active_response(
        &self,
        project_id: Uuid,
        method: &str,
        path: &str,
    ) -> RepositoryResult<Option<ActiveMockResponse>> {
        let mut conn = self.conn().await?;
        let Some(route_id) = conn
            .get::<_, Option<String>>(route_index_key(project_id, &method.to_uppercase(), path))
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
        project_id: Uuid,
        id: Uuid,
        request: ConvertUnknownRequest,
    ) -> RepositoryResult<ConvertedUnknownRequest> {
        validate_convert_request(&request)?;

        let mut conn = self.conn().await?;
        ensure_project_exists(&mut conn, project_id).await?;
        let Some(mut unknown) = load_unknown(&mut conn, &unknown_key(id)).await? else {
            return Err(RepositoryError::NotFound);
        };
        if unknown.project_id != project_id {
            return Err(RepositoryError::NotFound);
        }

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

        let route_index = route_index_key(project_id, &unknown.method, &unknown.path);
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
            project_id,
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
            profile_kind: request.scenario.profile_kind,
            kind: request.scenario.kind,
            proxy_url: request.scenario.proxy_url,
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
                route_profiles_key(route_id),
                scenario_id.to_string(),
                scenario.created_at.timestamp_millis(),
            )
            .zadd(
                route_set_key(project_id),
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
    validate_profile_request(&request.scenario)
}

fn validate_project_request(request: &CreateProject) -> RepositoryResult<()> {
    if request.name.trim().is_empty() {
        return Err(RepositoryError::Validation(
            "project.name cannot be empty".to_string(),
        ));
    }
    Ok(())
}

fn normalize_project_key(key: &str) -> RepositoryResult<String> {
    let key = key.trim().to_ascii_lowercase();
    if is_valid_project_key(&key) {
        Ok(key)
    } else {
        Err(RepositoryError::Validation(
            "project key must be 3-32 chars and contain only lowercase letters, numbers, and hyphens"
                .to_string(),
        ))
    }
}

fn is_valid_project_key(key: &str) -> bool {
    (3..=32).contains(&key.len())
        && key
            .bytes()
            .all(|byte| matches!(byte, b'a'..=b'z' | b'0'..=b'9' | b'-'))
        && key
            .bytes()
            .next()
            .is_some_and(|byte| matches!(byte, b'a'..=b'z' | b'0'..=b'9'))
}

fn validate_route_request(request: &UpsertRoute) -> RepositoryResult<()> {
    if !is_valid_http_method(&request.method) {
        return Err(RepositoryError::Validation(
            "route.method must be a valid HTTP method token".to_string(),
        ));
    }
    if !request.path_pattern.starts_with('/') {
        return Err(RepositoryError::Validation(
            "route.path_pattern must start with /".to_string(),
        ));
    }
    if request.path_pattern == "/mockadmin"
        || request.path_pattern.starts_with("/mockadmin/")
        || request.path_pattern == "/mockadminapi"
        || request.path_pattern.starts_with("/mockadminapi/")
    {
        return Err(RepositoryError::Validation(
            "admin paths cannot be used as mock routes".to_string(),
        ));
    }
    if request.name.trim().is_empty() {
        return Err(RepositoryError::Validation(
            "route.name cannot be empty".to_string(),
        ));
    }
    Ok(())
}

fn validate_profile_request(request: &CreateScenario) -> RepositoryResult<()> {
    if request.profile_kind == ProfileKind::Dynamic {
        let proxy_url = request.proxy_url.as_deref().unwrap_or_default();
        if !(proxy_url.starts_with("http://") || proxy_url.starts_with("https://")) {
            return Err(RepositoryError::Validation(
                "profile.proxy_url must be an http(s) URL for dynamic profiles".to_string(),
            ));
        }
    }

    if !(100..=599).contains(&request.status_code) {
        return Err(RepositoryError::Validation(
            "scenario.status_code must be between 100 and 599".to_string(),
        ));
    }

    if request.delay_ms < 0 {
        return Err(RepositoryError::Validation(
            "scenario.delay_ms must be non-negative".to_string(),
        ));
    }

    if !request.response_headers.is_object() {
        return Err(RepositoryError::Validation(
            "scenario.response_headers must be a JSON object".to_string(),
        ));
    }

    if !request.selection_rules.is_object() {
        return Err(RepositoryError::Validation(
            "scenario.selection_rules must be a JSON object".to_string(),
        ));
    }

    Ok(())
}

const PROJECT_SET: &str = "mock-machine:projects";

fn unknown_key(id: Uuid) -> String {
    format!("mock-machine:unknown:{id}")
}

fn unknown_index_key(project_id: Uuid, method: &str, path: &str) -> String {
    format!("mock-machine:unknown-index:{project_id}:{method}:{path}")
}

fn unknown_set_key(project_id: Uuid) -> String {
    format!("mock-machine:unknown:{project_id}")
}

fn route_key(id: Uuid) -> String {
    format!("mock-machine:route:{id}")
}

fn route_index_key(project_id: Uuid, method: &str, path: &str) -> String {
    format!("mock-machine:route-index:{project_id}:{method}:{path}")
}

fn route_set_key(project_id: Uuid) -> String {
    format!("mock-machine:routes:{project_id}")
}

fn project_key(id: Uuid) -> String {
    format!("mock-machine:project:{id}")
}

fn project_name_index_key(name: &str) -> String {
    format!("mock-machine:project-name:{}", name.to_ascii_lowercase())
}

fn project_key_index_key(key: &str) -> String {
    format!("mock-machine:project-key:{}", key.to_ascii_lowercase())
}

fn scenario_key(id: Uuid) -> String {
    format!("mock-machine:scenario:{id}")
}

fn route_profiles_key(route_id: Uuid) -> String {
    format!("mock-machine:route-profiles:{route_id}")
}

fn object_key_key(object_key: &str) -> String {
    format!("mock-machine:object:{object_key}")
}

async fn ensure_project_exists(
    conn: &mut redis::aio::MultiplexedConnection,
    project_id: Uuid,
) -> RepositoryResult<()> {
    ensure_default_project(conn).await?;
    if conn.exists::<_, bool>(project_key(project_id)).await? {
        Ok(())
    } else {
        Err(RepositoryError::NotFound)
    }
}

async fn ensure_default_project(
    conn: &mut redis::aio::MultiplexedConnection,
) -> RepositoryResult<Project> {
    let name_index = project_name_index_key("Default");
    if let Some(id) = conn.get::<_, Option<String>>(&name_index).await? {
        let id = Uuid::parse_str(&id).context("invalid default project id in Redis")?;
        if let Some(project) = load_json::<Project>(conn, &project_key(id)).await? {
            return ensure_project_key(conn, project).await;
        }
    }

    let now = Utc::now();
    let project = Project {
        id: Uuid::new_v4(),
        name: "Default".to_string(),
        key: "default".to_string(),
        created_at: now,
        updated_at: now,
    };
    let _: () = redis::pipe()
        .set(project_key(project.id), serde_json::to_string(&project)?)
        .set(name_index, project.id.to_string())
        .set(project_key_index_key(&project.key), project.id.to_string())
        .zadd(
            PROJECT_SET,
            project.id.to_string(),
            project.created_at.timestamp_millis(),
        )
        .query_async(conn)
        .await?;
    Ok(project)
}

async fn ensure_project_key(
    conn: &mut redis::aio::MultiplexedConnection,
    mut project: Project,
) -> RepositoryResult<Project> {
    if !project.key.is_empty() {
        let _: () = conn
            .set(project_key_index_key(&project.key), project.id.to_string())
            .await?;
        return Ok(project);
    }

    project.key = if project.name == "Default" {
        "default".to_string()
    } else {
        generate_project_key(conn).await?
    };
    project.updated_at = Utc::now();
    let _: () = redis::pipe()
        .set(project_key(project.id), serde_json::to_string(&project)?)
        .set(project_key_index_key(&project.key), project.id.to_string())
        .query_async(conn)
        .await?;
    Ok(project)
}

async fn generate_project_key(
    conn: &mut redis::aio::MultiplexedConnection,
) -> RepositoryResult<String> {
    for length in 8..=32 {
        for _ in 0..32 {
            let raw = Uuid::new_v4().simple().to_string();
            let candidate = raw[..length].to_string();
            if !conn
                .exists::<_, bool>(project_key_index_key(&candidate))
                .await?
            {
                return Ok(candidate);
            }
        }
    }

    Err(RepositoryError::Internal(anyhow::anyhow!(
        "failed to generate unique project key"
    )))
}
