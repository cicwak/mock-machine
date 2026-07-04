use anyhow::Context;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, ConnectionTrait, DatabaseConnection,
    DatabaseTransaction, EntityTrait, QueryFilter, QueryOrder, QuerySelect, Statement,
    TransactionTrait,
};
use uuid::Uuid;

use crate::{
    domain::{
        ActiveMockResponse, CapturedRequest, ConvertUnknownRequest, ConvertedUnknownRequest,
        CreateScenario, MockRoute, ProfileKind, ResponseScenario, ScenarioKind, UnknownRequest,
        UnknownRequestStatus, UpsertRoute,
    },
    entities::{mock_routes, response_scenarios, unknown_requests},
    repository::{
        MockRouteRepository, RepositoryError, RepositoryResult, UnknownRequestRepository,
    },
};

#[derive(Clone)]
pub struct PostgresRepository {
    db: DatabaseConnection,
}

impl PostgresRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl UnknownRequestRepository for PostgresRepository {
    async fn capture(&self, request: CapturedRequest) -> RepositoryResult<UnknownRequest> {
        capture_unknown(&self.db, request).await
    }

    async fn list(
        &self,
        status: Option<UnknownRequestStatus>,
        limit: u64,
    ) -> RepositoryResult<Vec<UnknownRequest>> {
        let mut query = unknown_requests::Entity::find()
            .order_by_desc(unknown_requests::Column::LastSeenAt)
            .limit(limit);

        if let Some(status) = status {
            query = query.filter(unknown_requests::Column::Status.eq(to_db_unknown_status(status)));
        }

        query
            .all(&self.db)
            .await
            .context("failed to list unknown requests")?
            .into_iter()
            .map(UnknownRequest::try_from)
            .collect()
    }

    async fn get(&self, id: Uuid) -> RepositoryResult<Option<UnknownRequest>> {
        let model = unknown_requests::Entity::find_by_id(id)
            .one(&self.db)
            .await
            .context("failed to get unknown request")?;

        model.map(UnknownRequest::try_from).transpose()
    }
}

#[async_trait::async_trait]
impl MockRouteRepository for PostgresRepository {
    async fn list_routes(&self) -> RepositoryResult<Vec<MockRoute>> {
        mock_routes::Entity::find()
            .order_by_asc(mock_routes::Column::PathPattern)
            .all(&self.db)
            .await
            .context("failed to list routes")?
            .into_iter()
            .map(MockRoute::try_from)
            .collect()
    }

    async fn get_route(&self, id: Uuid) -> RepositoryResult<Option<MockRoute>> {
        let model = mock_routes::Entity::find_by_id(id)
            .one(&self.db)
            .await
            .context("failed to get route")?;

        model.map(MockRoute::try_from).transpose()
    }

    async fn upsert_route(
        &self,
        id: Option<Uuid>,
        request: UpsertRoute,
    ) -> RepositoryResult<MockRoute> {
        validate_route_request(&request)?;

        if let Some(id) = id {
            let route = mock_routes::Entity::find_by_id(id)
                .one(&self.db)
                .await
                .context("failed to load route")?
                .ok_or(RepositoryError::NotFound)?;

            let mut update: mock_routes::ActiveModel = route.into();
            update.method = Set(request.method.to_uppercase());
            update.path_pattern = Set(request.path_pattern);
            update.name = Set(request.name);
            update.tags = Set(request.tags);
            update.status = Set(to_db_route_status(request.status));
            update.active_scenario_id = Set(request.active_scenario_id);

            return update_route(update, &self.db).await;
        }

        let route = mock_routes::ActiveModel {
            method: Set(request.method.to_uppercase()),
            path_pattern: Set(request.path_pattern),
            name: Set(request.name),
            tags: Set(request.tags),
            status: Set(to_db_route_status(request.status)),
            active_scenario_id: Set(request.active_scenario_id),
            ..Default::default()
        };

        insert_route(route, &self.db).await
    }

    async fn list_profiles(&self, route_id: Uuid) -> RepositoryResult<Vec<ResponseScenario>> {
        ensure_route_exists(&self.db, route_id).await?;

        response_scenarios::Entity::find()
            .filter(response_scenarios::Column::RouteId.eq(route_id))
            .order_by_asc(response_scenarios::Column::Name)
            .all(&self.db)
            .await
            .context("failed to list route profiles")?
            .into_iter()
            .map(ResponseScenario::try_from)
            .collect()
    }

    async fn upsert_profile(
        &self,
        route_id: Uuid,
        profile_id: Option<Uuid>,
        request: CreateScenario,
    ) -> RepositoryResult<ResponseScenario> {
        ensure_route_exists(&self.db, route_id).await?;
        validate_profile_request(&request)?;

        if let Some(profile_id) = profile_id {
            let profile = response_scenarios::Entity::find_by_id(profile_id)
                .filter(response_scenarios::Column::RouteId.eq(route_id))
                .one(&self.db)
                .await
                .context("failed to load route profile")?
                .ok_or(RepositoryError::NotFound)?;

            let mut update: response_scenarios::ActiveModel = profile.into();
            fill_profile_model(&mut update, request);
            return update
                .update(&self.db)
                .await
                .context("failed to update route profile")?
                .try_into();
        }

        response_scenarios::ActiveModel {
            route_id: Set(route_id),
            name: Set(request.name),
            profile_kind: Set(to_db_profile_kind(request.profile_kind)),
            kind: Set(to_db_scenario_kind(request.kind)),
            proxy_url: Set(request.proxy_url),
            status_code: Set(request.status_code),
            response_headers: Set(request.response_headers),
            response_body: Set(request.response_body),
            delay_ms: Set(request.delay_ms),
            selection_rules: Set(request.selection_rules),
            ..Default::default()
        }
        .insert(&self.db)
        .await
        .map_err(map_profile_insert_error)?
        .try_into()
    }

    async fn set_active_profile(
        &self,
        route_id: Uuid,
        profile_id: Uuid,
    ) -> RepositoryResult<MockRoute> {
        let route = mock_routes::Entity::find_by_id(route_id)
            .one(&self.db)
            .await
            .context("failed to load route")?
            .ok_or(RepositoryError::NotFound)?;

        let profile_exists = response_scenarios::Entity::find_by_id(profile_id)
            .filter(response_scenarios::Column::RouteId.eq(route_id))
            .one(&self.db)
            .await
            .context("failed to load route profile")?
            .is_some();
        if !profile_exists {
            return Err(RepositoryError::NotFound);
        }

        let mut update: mock_routes::ActiveModel = route.into();
        update.active_scenario_id = Set(Some(profile_id));
        update
            .update(&self.db)
            .await
            .context("failed to set active route profile")?
            .try_into()
    }

    async fn find_active_response(
        &self,
        method: &str,
        path: &str,
    ) -> RepositoryResult<Option<ActiveMockResponse>> {
        let Some(route) = mock_routes::Entity::find()
            .filter(mock_routes::Column::Method.eq(method.to_uppercase()))
            .filter(mock_routes::Column::PathPattern.eq(path))
            .filter(mock_routes::Column::Status.eq(mock_routes::RouteStatus::Active))
            .one(&self.db)
            .await
            .context("failed to find mock route")?
        else {
            return Ok(None);
        };

        let Some(scenario_id) = route.active_scenario_id else {
            return Ok(None);
        };

        let Some(scenario) = response_scenarios::Entity::find_by_id(scenario_id)
            .one(&self.db)
            .await
            .context("failed to load active scenario")?
        else {
            return Ok(None);
        };

        Ok(Some(ActiveMockResponse {
            route: route.try_into()?,
            scenario: scenario.try_into()?,
        }))
    }

    async fn convert_unknown_request(
        &self,
        id: Uuid,
        request: ConvertUnknownRequest,
    ) -> RepositoryResult<ConvertedUnknownRequest> {
        let tx = self
            .db
            .begin()
            .await
            .context("failed to begin conversion transaction")?;

        let result = convert_unknown_request_tx(&tx, id, request).await;

        match result {
            Ok(converted) => {
                tx.commit()
                    .await
                    .context("failed to commit conversion transaction")?;
                Ok(converted)
            }
            Err(error) => {
                tx.rollback()
                    .await
                    .context("failed to rollback conversion transaction")?;
                Err(error)
            }
        }
    }
}

async fn capture_unknown<C>(db: &C, request: CapturedRequest) -> RepositoryResult<UnknownRequest>
where
    C: ConnectionTrait,
{
    let query = serde_json::to_string(&request.query).context("failed to encode query JSON")?;
    let headers =
        serde_json::to_string(&request.headers).context("failed to encode header JSON")?;

    let sql = r#"
        INSERT INTO unknown_requests (method, path, query, headers, body)
        VALUES ($1, $2, $3::jsonb, $4::jsonb, $5)
        ON CONFLICT (method, path) DO UPDATE
        SET
            query = EXCLUDED.query,
            headers = EXCLUDED.headers,
            body = EXCLUDED.body,
            last_seen_at = now(),
            count = unknown_requests.count + 1,
            status = CASE
                WHEN unknown_requests.status = 'converted' THEN unknown_requests.status
                ELSE 'new'::unknown_request_status
            END
        RETURNING
            id, method, path, query, headers, body, first_seen_at, last_seen_at,
            count, status::text AS status, converted_route_id
    "#;

    let row = db
        .query_one(Statement::from_sql_and_values(
            db.get_database_backend(),
            sql,
            vec![
                request.method.to_uppercase().into(),
                request.path.into(),
                query.into(),
                headers.into(),
                request.body.into(),
            ],
        ))
        .await
        .context("failed to upsert unknown request")?
        .ok_or_else(|| anyhow::anyhow!("unknown request upsert returned no row"))?;

    unknown_from_row(&row)
}

async fn convert_unknown_request_tx(
    tx: &DatabaseTransaction,
    id: Uuid,
    request: ConvertUnknownRequest,
) -> RepositoryResult<ConvertedUnknownRequest> {
    validate_convert_request(&request)?;

    let unknown = unknown_requests::Entity::find_by_id(id)
        .one(tx)
        .await
        .context("failed to load unknown request")?
        .ok_or(RepositoryError::NotFound)?;

    if unknown.status == unknown_requests::UnknownRequestStatus::Converted {
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

    let route_name = request.name.unwrap_or_else(|| {
        format!(
            "{} {}",
            unknown.method,
            unknown.path.trim_start_matches('/').replace('/', " / ")
        )
    });

    let route = mock_routes::ActiveModel {
        method: Set(unknown.method.clone()),
        path_pattern: Set(unknown.path.clone()),
        name: Set(route_name),
        tags: Set(request.tags),
        status: Set(mock_routes::RouteStatus::Active),
        ..Default::default()
    }
    .insert(tx)
    .await
    .map_err(|error| {
        if error
            .to_string()
            .contains("mock_routes_method_path_pattern_key")
        {
            RepositoryError::Conflict("route already exists for this method and path".to_string())
        } else {
            RepositoryError::Internal(error.into())
        }
    })?;

    let scenario = response_scenarios::ActiveModel {
        route_id: Set(route.id),
        name: Set(request.scenario.name),
        profile_kind: Set(to_db_profile_kind(request.scenario.profile_kind)),
        kind: Set(to_db_scenario_kind(request.scenario.kind)),
        proxy_url: Set(request.scenario.proxy_url),
        status_code: Set(request.scenario.status_code),
        response_headers: Set(request.scenario.response_headers),
        response_body: Set(request.scenario.response_body),
        delay_ms: Set(request.scenario.delay_ms),
        selection_rules: Set(request.scenario.selection_rules),
        ..Default::default()
    }
    .insert(tx)
    .await
    .context("failed to insert response scenario")?;

    let mut route_update: mock_routes::ActiveModel = route.clone().into();
    route_update.active_scenario_id = Set(Some(scenario.id));
    let route = route_update
        .update(tx)
        .await
        .context("failed to set active scenario")?;

    let mut unknown_update: unknown_requests::ActiveModel = unknown.into();
    unknown_update.status = Set(unknown_requests::UnknownRequestStatus::Converted);
    unknown_update.converted_route_id = Set(Some(route.id));
    let unknown = unknown_update
        .update(tx)
        .await
        .context("failed to mark unknown request as converted")?;

    Ok(ConvertedUnknownRequest {
        route: route.try_into()?,
        scenario: scenario.try_into()?,
        unknown_request: unknown.try_into()?,
    })
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

fn validate_route_request(request: &UpsertRoute) -> RepositoryResult<()> {
    if request.method.trim().is_empty() {
        return Err(RepositoryError::Validation(
            "route.method cannot be empty".to_string(),
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

fn unknown_from_row(row: &sea_orm::QueryResult) -> RepositoryResult<UnknownRequest> {
    let status: String = row.try_get("", "status").context("missing status")?;
    Ok(UnknownRequest {
        id: row.try_get("", "id").context("missing id")?,
        method: row.try_get("", "method").context("missing method")?,
        path: row.try_get("", "path").context("missing path")?,
        query: row.try_get("", "query").context("missing query")?,
        headers: row.try_get("", "headers").context("missing headers")?,
        body: row.try_get("", "body").context("missing body")?,
        first_seen_at: row
            .try_get("", "first_seen_at")
            .context("missing first_seen_at")?,
        last_seen_at: row
            .try_get("", "last_seen_at")
            .context("missing last_seen_at")?,
        count: row.try_get("", "count").context("missing count")?,
        status: parse_unknown_status(&status)?,
        converted_route_id: row
            .try_get("", "converted_route_id")
            .context("missing converted_route_id")?,
    })
}

impl TryFrom<unknown_requests::Model> for UnknownRequest {
    type Error = RepositoryError;

    fn try_from(value: unknown_requests::Model) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.id,
            method: value.method,
            path: value.path,
            query: value.query,
            headers: value.headers,
            body: value.body,
            first_seen_at: value.first_seen_at,
            last_seen_at: value.last_seen_at,
            count: value.count,
            status: from_db_unknown_status(value.status),
            converted_route_id: value.converted_route_id,
        })
    }
}

impl TryFrom<mock_routes::Model> for MockRoute {
    type Error = RepositoryError;

    fn try_from(value: mock_routes::Model) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.id,
            method: value.method,
            path_pattern: value.path_pattern,
            name: value.name,
            tags: value.tags,
            status: match value.status {
                mock_routes::RouteStatus::Active => crate::domain::RouteStatus::Active,
                mock_routes::RouteStatus::Disabled => crate::domain::RouteStatus::Disabled,
            },
            active_scenario_id: value.active_scenario_id,
            created_at: value.created_at,
            updated_at: value.updated_at,
        })
    }
}

impl TryFrom<response_scenarios::Model> for ResponseScenario {
    type Error = RepositoryError;

    fn try_from(value: response_scenarios::Model) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.id,
            route_id: value.route_id,
            name: value.name,
            profile_kind: from_db_profile_kind(value.profile_kind),
            kind: from_db_scenario_kind(value.kind),
            proxy_url: value.proxy_url,
            status_code: value.status_code,
            response_headers: value.response_headers,
            response_body: value.response_body,
            delay_ms: value.delay_ms,
            selection_rules: value.selection_rules,
            created_at: value.created_at,
            updated_at: value.updated_at,
        })
    }
}

async fn ensure_route_exists<C>(db: &C, route_id: Uuid) -> RepositoryResult<()>
where
    C: ConnectionTrait,
{
    let exists = mock_routes::Entity::find_by_id(route_id)
        .one(db)
        .await
        .context("failed to load route")?
        .is_some();
    if exists {
        Ok(())
    } else {
        Err(RepositoryError::NotFound)
    }
}

async fn insert_route<C>(route: mock_routes::ActiveModel, db: &C) -> RepositoryResult<MockRoute>
where
    C: ConnectionTrait,
{
    route
        .insert(db)
        .await
        .map_err(map_route_write_error)?
        .try_into()
}

async fn update_route<C>(route: mock_routes::ActiveModel, db: &C) -> RepositoryResult<MockRoute>
where
    C: ConnectionTrait,
{
    route
        .update(db)
        .await
        .map_err(map_route_write_error)?
        .try_into()
}

fn map_route_write_error(error: sea_orm::DbErr) -> RepositoryError {
    if error
        .to_string()
        .contains("mock_routes_method_path_pattern_key")
    {
        RepositoryError::Conflict("route already exists for this method and path".to_string())
    } else {
        RepositoryError::Internal(error.into())
    }
}

fn map_profile_insert_error(error: sea_orm::DbErr) -> RepositoryError {
    if error
        .to_string()
        .contains("response_scenarios_route_name_key")
    {
        RepositoryError::Conflict("profile already exists for this route and name".to_string())
    } else {
        RepositoryError::Internal(error.into())
    }
}

fn fill_profile_model(model: &mut response_scenarios::ActiveModel, request: CreateScenario) {
    model.name = Set(request.name);
    model.profile_kind = Set(to_db_profile_kind(request.profile_kind));
    model.kind = Set(to_db_scenario_kind(request.kind));
    model.proxy_url = Set(request.proxy_url);
    model.status_code = Set(request.status_code);
    model.response_headers = Set(request.response_headers);
    model.response_body = Set(request.response_body);
    model.delay_ms = Set(request.delay_ms);
    model.selection_rules = Set(request.selection_rules);
}

fn to_db_route_status(status: crate::domain::RouteStatus) -> mock_routes::RouteStatus {
    match status {
        crate::domain::RouteStatus::Active => mock_routes::RouteStatus::Active,
        crate::domain::RouteStatus::Disabled => mock_routes::RouteStatus::Disabled,
    }
}

fn to_db_profile_kind(kind: ProfileKind) -> response_scenarios::ProfileKind {
    match kind {
        ProfileKind::Static => response_scenarios::ProfileKind::Static,
        ProfileKind::Dynamic => response_scenarios::ProfileKind::Dynamic,
    }
}

fn from_db_profile_kind(kind: response_scenarios::ProfileKind) -> ProfileKind {
    match kind {
        response_scenarios::ProfileKind::Static => ProfileKind::Static,
        response_scenarios::ProfileKind::Dynamic => ProfileKind::Dynamic,
    }
}

fn to_db_unknown_status(status: UnknownRequestStatus) -> unknown_requests::UnknownRequestStatus {
    match status {
        UnknownRequestStatus::New => unknown_requests::UnknownRequestStatus::New,
        UnknownRequestStatus::Ignored => unknown_requests::UnknownRequestStatus::Ignored,
        UnknownRequestStatus::Converted => unknown_requests::UnknownRequestStatus::Converted,
    }
}

fn from_db_unknown_status(status: unknown_requests::UnknownRequestStatus) -> UnknownRequestStatus {
    match status {
        unknown_requests::UnknownRequestStatus::New => UnknownRequestStatus::New,
        unknown_requests::UnknownRequestStatus::Ignored => UnknownRequestStatus::Ignored,
        unknown_requests::UnknownRequestStatus::Converted => UnknownRequestStatus::Converted,
    }
}

fn to_db_scenario_kind(kind: ScenarioKind) -> response_scenarios::ScenarioKind {
    match kind {
        ScenarioKind::Success => response_scenarios::ScenarioKind::Success,
        ScenarioKind::Error => response_scenarios::ScenarioKind::Error,
        ScenarioKind::Timeout => response_scenarios::ScenarioKind::Timeout,
        ScenarioKind::Custom => response_scenarios::ScenarioKind::Custom,
    }
}

fn from_db_scenario_kind(kind: response_scenarios::ScenarioKind) -> ScenarioKind {
    match kind {
        response_scenarios::ScenarioKind::Success => ScenarioKind::Success,
        response_scenarios::ScenarioKind::Error => ScenarioKind::Error,
        response_scenarios::ScenarioKind::Timeout => ScenarioKind::Timeout,
        response_scenarios::ScenarioKind::Custom => ScenarioKind::Custom,
    }
}

fn parse_unknown_status(value: &str) -> RepositoryResult<UnknownRequestStatus> {
    match value {
        "new" => Ok(UnknownRequestStatus::New),
        "ignored" => Ok(UnknownRequestStatus::Ignored),
        "converted" => Ok(UnknownRequestStatus::Converted),
        other => Err(RepositoryError::Internal(anyhow::anyhow!(
            "unknown request status returned from database: {other}"
        ))),
    }
}
