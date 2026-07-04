use std::collections::HashMap;

use bytes::Bytes;
use chrono::Utc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::{
    domain::{
        ActiveMockResponse, CapturedRequest, ConvertUnknownRequest, ConvertedUnknownRequest,
        CreateScenario, MockRoute, ObjectAsset, ProfileKind, ResponseScenario, RouteStatus,
        UnknownRequest, UnknownRequestStatus, UpsertRoute,
    },
    repository::{
        MockRouteRepository, ObjectAssetRepository, RepositoryError, RepositoryResult,
        UnknownRequestRepository,
    },
};

#[derive(Default)]
pub struct InMemoryRepository {
    unknown: RwLock<HashMap<Uuid, UnknownRequest>>,
    unknown_index: RwLock<HashMap<(String, String), Uuid>>,
    routes: RwLock<HashMap<Uuid, MockRoute>>,
    scenarios: RwLock<HashMap<Uuid, ResponseScenario>>,
    objects: RwLock<HashMap<String, (Option<String>, Bytes)>>,
}

#[async_trait::async_trait]
impl UnknownRequestRepository for InMemoryRepository {
    async fn capture(&self, request: CapturedRequest) -> RepositoryResult<UnknownRequest> {
        let key = (request.method.to_uppercase(), request.path.clone());
        let now = Utc::now();

        let id = {
            let mut index = self.unknown_index.write().await;
            *index.entry(key.clone()).or_insert_with(Uuid::new_v4)
        };

        let mut unknown = self.unknown.write().await;
        let entry = unknown.entry(id).or_insert_with(|| UnknownRequest {
            id,
            method: key.0.clone(),
            path: key.1.clone(),
            query: request.query.clone(),
            headers: request.headers.clone(),
            body: request.body.clone(),
            first_seen_at: now,
            last_seen_at: now,
            count: 0,
            status: UnknownRequestStatus::New,
            converted_route_id: None,
        });

        entry.query = request.query;
        entry.headers = request.headers;
        entry.body = request.body;
        entry.last_seen_at = now;
        entry.count += 1;
        if entry.status != UnknownRequestStatus::Converted {
            entry.status = UnknownRequestStatus::New;
        }

        Ok(entry.clone())
    }

    async fn list(
        &self,
        status: Option<UnknownRequestStatus>,
        limit: u64,
    ) -> RepositoryResult<Vec<UnknownRequest>> {
        let mut values = self
            .unknown
            .read()
            .await
            .values()
            .filter(|request| {
                status
                    .as_ref()
                    .is_none_or(|expected| request.status == *expected)
            })
            .cloned()
            .collect::<Vec<_>>();

        values.sort_by(|left, right| right.last_seen_at.cmp(&left.last_seen_at));
        values.truncate(limit as usize);
        Ok(values)
    }

    async fn get(&self, id: Uuid) -> RepositoryResult<Option<UnknownRequest>> {
        Ok(self.unknown.read().await.get(&id).cloned())
    }
}

#[async_trait::async_trait]
impl MockRouteRepository for InMemoryRepository {
    async fn list_routes(&self) -> RepositoryResult<Vec<MockRoute>> {
        let mut routes = self
            .routes
            .read()
            .await
            .values()
            .cloned()
            .collect::<Vec<_>>();
        routes.sort_by(|left, right| left.path_pattern.cmp(&right.path_pattern));
        Ok(routes)
    }

    async fn get_route(&self, id: Uuid) -> RepositoryResult<Option<MockRoute>> {
        Ok(self.routes.read().await.get(&id).cloned())
    }

    async fn upsert_route(
        &self,
        id: Option<Uuid>,
        request: UpsertRoute,
    ) -> RepositoryResult<MockRoute> {
        let id = id.unwrap_or_else(Uuid::new_v4);
        let now = Utc::now();
        let mut routes = self.routes.write().await;
        let created_at = routes.get(&id).map(|route| route.created_at).unwrap_or(now);
        let route = MockRoute {
            id,
            method: request.method.to_uppercase(),
            path_pattern: request.path_pattern,
            name: request.name,
            tags: request.tags,
            status: request.status,
            active_scenario_id: request.active_scenario_id,
            created_at,
            updated_at: now,
        };
        routes.insert(id, route.clone());
        Ok(route)
    }

    async fn list_profiles(&self, route_id: Uuid) -> RepositoryResult<Vec<ResponseScenario>> {
        if !self.routes.read().await.contains_key(&route_id) {
            return Err(RepositoryError::NotFound);
        }
        let mut profiles = self
            .scenarios
            .read()
            .await
            .values()
            .filter(|profile| profile.route_id == route_id)
            .cloned()
            .collect::<Vec<_>>();
        profiles.sort_by(|left, right| left.name.cmp(&right.name));
        Ok(profiles)
    }

    async fn upsert_profile(
        &self,
        route_id: Uuid,
        profile_id: Option<Uuid>,
        request: CreateScenario,
    ) -> RepositoryResult<ResponseScenario> {
        if !self.routes.read().await.contains_key(&route_id) {
            return Err(RepositoryError::NotFound);
        }
        validate_profile_request(&request)?;

        let id = profile_id.unwrap_or_else(Uuid::new_v4);
        let now = Utc::now();
        let mut scenarios = self.scenarios.write().await;
        let created_at = scenarios
            .get(&id)
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
        scenarios.insert(id, profile.clone());
        Ok(profile)
    }

    async fn set_active_profile(
        &self,
        route_id: Uuid,
        profile_id: Uuid,
    ) -> RepositoryResult<MockRoute> {
        let profile_exists = self
            .scenarios
            .read()
            .await
            .get(&profile_id)
            .is_some_and(|profile| profile.route_id == route_id);
        if !profile_exists {
            return Err(RepositoryError::NotFound);
        }
        let mut routes = self.routes.write().await;
        let route = routes.get_mut(&route_id).ok_or(RepositoryError::NotFound)?;
        route.active_scenario_id = Some(profile_id);
        route.updated_at = Utc::now();
        Ok(route.clone())
    }

    async fn find_active_response(
        &self,
        method: &str,
        path: &str,
    ) -> RepositoryResult<Option<ActiveMockResponse>> {
        let route = self
            .routes
            .read()
            .await
            .values()
            .find(|route| {
                route.method == method.to_uppercase()
                    && route.path_pattern == path
                    && route.status == RouteStatus::Active
            })
            .cloned();

        let Some(route) = route else {
            return Ok(None);
        };

        let Some(scenario_id) = route.active_scenario_id else {
            return Ok(None);
        };

        let scenario = self.scenarios.read().await.get(&scenario_id).cloned();
        Ok(scenario.map(|scenario| ActiveMockResponse { route, scenario }))
    }

    async fn convert_unknown_request(
        &self,
        id: Uuid,
        request: ConvertUnknownRequest,
    ) -> RepositoryResult<ConvertedUnknownRequest> {
        validate_convert_request(&request)?;

        let mut unknown_requests = self.unknown.write().await;
        let unknown = unknown_requests
            .get_mut(&id)
            .ok_or(RepositoryError::NotFound)?;

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

        let duplicate = self
            .routes
            .read()
            .await
            .values()
            .any(|route| route.method == unknown.method && route.path_pattern == unknown.path);
        if duplicate {
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

        self.routes.write().await.insert(route_id, route.clone());
        self.scenarios
            .write()
            .await
            .insert(scenario_id, scenario.clone());

        unknown.status = UnknownRequestStatus::Converted;
        unknown.converted_route_id = Some(route_id);

        Ok(ConvertedUnknownRequest {
            route,
            scenario,
            unknown_request: unknown.clone(),
        })
    }
}

#[async_trait::async_trait]
impl ObjectAssetRepository for InMemoryRepository {
    async fn put(
        &self,
        object_key: &str,
        content_type: Option<&str>,
        body: Bytes,
    ) -> RepositoryResult<ObjectAsset> {
        self.objects.write().await.insert(
            object_key.to_string(),
            (content_type.map(ToOwned::to_owned), body.clone()),
        );

        Ok(ObjectAsset {
            bucket: "in-memory".to_string(),
            object_key: object_key.to_string(),
            content_type: content_type.map(ToOwned::to_owned),
            size_bytes: body.len() as i64,
        })
    }

    async fn get(&self, object_key: &str) -> RepositoryResult<Option<Bytes>> {
        Ok(self
            .objects
            .read()
            .await
            .get(object_key)
            .map(|(_, body)| body.clone()))
    }
}

fn validate_convert_request(request: &ConvertUnknownRequest) -> RepositoryResult<()> {
    validate_profile_request(&request.scenario)
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
