use axum::{
    Json, Router,
    body::Body,
    extract::{OriginalUri, Path, Query, State},
    http::{HeaderMap, HeaderName, HeaderValue, Method, StatusCode, Uri},
    response::{IntoResponse, Response},
    routing::{get, post, put},
};
use base64::{Engine, engine::general_purpose::STANDARD};
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use tokio::time::{Duration, sleep};
use tracing::{info, warn};
use url::form_urlencoded;
use uuid::Uuid;

use crate::{
    domain::{
        ActiveMockResponse, CapturedRequest, ConvertUnknownRequest, CreateScenario, ProfileKind,
        RouteStatus, ScenarioKind, UnknownRequest, UnknownRequestStatus, UpsertRoute,
    },
    repository::RepositoryError,
    state::AppState,
};

#[derive(Serialize)]
struct HealthResponse<'a> {
    status: &'a str,
    service: &'a str,
    storage: &'a str,
}

#[derive(Debug, Deserialize)]
struct ListUnknownRequestsQuery {
    status: Option<UnknownRequestStatus>,
    #[serde(default = "default_unknown_limit")]
    limit: u64,
}

#[derive(Debug, Deserialize)]
struct ConvertUnknownRequestPayload {
    name: Option<String>,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    scenario: ConvertScenarioPayload,
}

#[derive(Debug, Deserialize)]
struct ConvertScenarioPayload {
    #[serde(default = "default_scenario_name")]
    name: String,
    #[serde(default = "default_profile_kind")]
    profile_kind: ProfileKind,
    #[serde(default = "default_scenario_kind")]
    kind: ScenarioKind,
    proxy_url: Option<String>,
    #[serde(default = "default_status_code")]
    status_code: i32,
    #[serde(default = "default_response_headers")]
    response_headers: Value,
    response_body: Option<String>,
    #[serde(default)]
    delay_ms: i32,
    #[serde(default = "default_selection_rules")]
    selection_rules: Value,
}

impl Default for ConvertScenarioPayload {
    fn default() -> Self {
        Self {
            name: default_scenario_name(),
            profile_kind: default_profile_kind(),
            kind: default_scenario_kind(),
            proxy_url: None,
            status_code: default_status_code(),
            response_headers: default_response_headers(),
            response_body: None,
            delay_ms: 0,
            selection_rules: default_selection_rules(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct RoutePayload {
    method: String,
    path_pattern: String,
    name: String,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default = "default_route_status")]
    status: RouteStatus,
    active_scenario_id: Option<Uuid>,
}

#[derive(Debug, Serialize)]
struct UnknownRequestResponse {
    #[serde(flatten)]
    request: UnknownRequest,
    body_base64: Option<String>,
    body_text: Option<String>,
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: String,
}

#[derive(Debug)]
pub enum AppError {
    NotFound(String),
    Conflict(String),
    BadRequest(String),
    Internal(anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::NotFound(message) => (StatusCode::NOT_FOUND, message),
            AppError::Conflict(message) => (StatusCode::CONFLICT, message),
            AppError::BadRequest(message) => (StatusCode::BAD_REQUEST, message),
            AppError::Internal(error) => {
                warn!(%error, "request failed");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal server error".to_string(),
                )
            }
        };

        (status, Json(ErrorResponse { error: message })).into_response()
    }
}

impl From<RepositoryError> for AppError {
    fn from(value: RepositoryError) -> Self {
        match value {
            RepositoryError::NotFound => AppError::NotFound("resource not found".to_string()),
            RepositoryError::Conflict(message) => AppError::Conflict(message),
            RepositoryError::Validation(message) => AppError::BadRequest(message),
            RepositoryError::Internal(error) => AppError::Internal(error),
        }
    }
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/mockadminapi/health", get(health))
        .route("/mockadminapi/routes", get(list_routes).post(create_route))
        .route(
            "/mockadminapi/routes/{id}",
            get(get_route).put(update_route),
        )
        .route(
            "/mockadminapi/routes/{id}/profiles",
            get(list_profiles).post(create_profile),
        )
        .route(
            "/mockadminapi/routes/{id}/profiles/{profile_id}",
            put(update_profile),
        )
        .route(
            "/mockadminapi/routes/{id}/active-profile/{profile_id}",
            put(set_active_profile),
        )
        .route("/mockadminapi/unknown-requests", get(list_unknown_requests))
        .route(
            "/mockadminapi/unknown-requests/{id}",
            get(get_unknown_request),
        )
        .route(
            "/mockadminapi/unknown-requests/{id}/convert",
            post(convert_unknown_request),
        )
        .route("/mockadminapi/assets/{*key}", put(put_asset).get(get_asset))
        .fallback(mock_fallback)
        .with_state(state)
}

async fn health(State(state): State<AppState>) -> Json<HealthResponse<'static>> {
    Json(HealthResponse {
        status: "ok",
        service: "mock-machine-backend",
        storage: state.storage,
    })
}

async fn list_routes(State(state): State<AppState>) -> Result<Json<Value>, AppError> {
    let routes = state.routes.list_routes().await?;
    Ok(Json(json!({ "items": routes })))
}

async fn get_route(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, AppError> {
    let route = state
        .routes
        .get_route(id)
        .await?
        .ok_or_else(|| AppError::NotFound("route not found".to_string()))?;
    Ok(Json(json!(route)))
}

async fn create_route(
    State(state): State<AppState>,
    Json(payload): Json<RoutePayload>,
) -> Result<Json<Value>, AppError> {
    let route = state.routes.upsert_route(None, payload.into()).await?;
    Ok(Json(json!(route)))
}

async fn update_route(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<RoutePayload>,
) -> Result<Json<Value>, AppError> {
    let route = state.routes.upsert_route(Some(id), payload.into()).await?;
    Ok(Json(json!(route)))
}

async fn list_profiles(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, AppError> {
    let profiles = state.routes.list_profiles(id).await?;
    Ok(Json(json!({ "items": profiles })))
}

async fn create_profile(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<ConvertScenarioPayload>,
) -> Result<Json<Value>, AppError> {
    let profile = state
        .routes
        .upsert_profile(id, None, payload.into())
        .await?;
    Ok(Json(json!(profile)))
}

async fn update_profile(
    State(state): State<AppState>,
    Path((id, profile_id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<ConvertScenarioPayload>,
) -> Result<Json<Value>, AppError> {
    let profile = state
        .routes
        .upsert_profile(id, Some(profile_id), payload.into())
        .await?;
    Ok(Json(json!(profile)))
}

async fn set_active_profile(
    State(state): State<AppState>,
    Path((id, profile_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<Value>, AppError> {
    let route = state.routes.set_active_profile(id, profile_id).await?;
    Ok(Json(json!(route)))
}

async fn list_unknown_requests(
    State(state): State<AppState>,
    Query(query): Query<ListUnknownRequestsQuery>,
) -> Result<Json<Value>, AppError> {
    let limit = query.limit.clamp(1, 200);
    let requests = state.unknown_requests.list(query.status, limit).await?;
    let items = requests
        .into_iter()
        .map(UnknownRequestResponse::from)
        .collect::<Vec<_>>();

    Ok(Json(json!({ "items": items })))
}

async fn get_unknown_request(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<UnknownRequestResponse>, AppError> {
    let request = state
        .unknown_requests
        .get(id)
        .await?
        .ok_or_else(|| AppError::NotFound("unknown request not found".to_string()))?;

    Ok(Json(request.into()))
}

async fn convert_unknown_request(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<ConvertUnknownRequestPayload>,
) -> Result<Json<Value>, AppError> {
    let converted = state
        .routes
        .convert_unknown_request(id, payload.into())
        .await?;

    Ok(Json(json!(converted)))
}

async fn put_asset(
    State(state): State<AppState>,
    Path(key): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Json<Value>, AppError> {
    if key.is_empty() {
        return Err(AppError::BadRequest(
            "asset key cannot be empty".to_string(),
        ));
    }

    let content_type = headers
        .get(axum::http::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok());
    let asset = state.assets.put(&key, content_type, body).await?;

    Ok(Json(json!(asset)))
}

async fn get_asset(
    State(state): State<AppState>,
    Path(key): Path<String>,
) -> Result<Response, AppError> {
    let body = state
        .assets
        .get(&key)
        .await?
        .ok_or_else(|| AppError::NotFound("asset not found".to_string()))?;

    Response::builder()
        .status(StatusCode::OK)
        .body(Body::from(body))
        .map_err(|error| AppError::Internal(error.into()))
}

async fn mock_fallback(
    State(state): State<AppState>,
    method: Method,
    OriginalUri(uri): OriginalUri,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Response, AppError> {
    let path = uri.path();

    if path == "/mockadminapi" || path.starts_with("/mockadminapi/") {
        return Err(AppError::NotFound("admin API route not found".to_string()));
    }

    if path == "/mockadmin" || path.starts_with("/mockadmin/") {
        return Err(AppError::NotFound(
            "admin frontend route not found".to_string(),
        ));
    }

    if let Some(active_response) = state
        .routes
        .find_active_response(method.as_str(), path)
        .await?
    {
        return render_mock_response(active_response, method, uri, headers, body).await;
    }

    let captured = CapturedRequest {
        method: method.as_str().to_string(),
        path: path.to_string(),
        query: query_to_json(uri.query()),
        headers: headers_to_json(&headers),
        body: (!body.is_empty()).then(|| body.to_vec()),
    };

    let saved = state.unknown_requests.capture(captured).await?;
    state.realtime.unknown_request_captured(saved.clone()).await;
    info!(
        id = %saved.id,
        method = %saved.method,
        path = %saved.path,
        count = saved.count,
        "captured unknown mock request"
    );

    Ok((StatusCode::NOT_FOUND, "route is not configured").into_response())
}

async fn render_mock_response(
    active_response: ActiveMockResponse,
    method: Method,
    uri: Uri,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Response, AppError> {
    let scenario = active_response.scenario;

    if scenario.delay_ms > 0 {
        sleep(Duration::from_millis(scenario.delay_ms as u64)).await;
    }

    if scenario.profile_kind == ProfileKind::Dynamic {
        return proxy_dynamic_response(scenario.proxy_url, method, uri, headers, body).await;
    }

    if scenario.kind == ScenarioKind::Timeout {
        if scenario.delay_ms == 0 {
            sleep(Duration::from_secs(30)).await;
        }
        return Ok((StatusCode::GATEWAY_TIMEOUT, "mock timeout").into_response());
    }

    let status = StatusCode::from_u16(scenario.status_code as u16)
        .map_err(|_| AppError::BadRequest("invalid scenario status code".to_string()))?;

    let mut builder = Response::builder().status(status);
    if let Some(headers) = scenario.response_headers.as_object() {
        for (name, value) in headers {
            let Some(value) = value.as_str() else {
                warn!(%name, "skipping non-string response header value");
                continue;
            };

            let Ok(header_name) = HeaderName::from_bytes(name.as_bytes()) else {
                warn!(%name, "skipping invalid response header name");
                continue;
            };

            let Ok(header_value) = HeaderValue::from_str(value) else {
                warn!(%name, "skipping invalid response header value");
                continue;
            };

            builder = builder.header(header_name, header_value);
        }
    }

    builder
        .body(Body::from(scenario.response_body.unwrap_or_default()))
        .map_err(|error| AppError::Internal(error.into()))
}

async fn proxy_dynamic_response(
    proxy_url: Option<String>,
    method: Method,
    uri: Uri,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Response, AppError> {
    let proxy_url = proxy_url.ok_or_else(|| {
        AppError::BadRequest("dynamic profile proxy_url is not configured".to_string())
    })?;
    let target = build_proxy_url(&proxy_url, &uri)?;
    let client = reqwest::Client::new();
    let reqwest_method = reqwest::Method::from_bytes(method.as_str().as_bytes())
        .map_err(|error| AppError::Internal(error.into()))?;
    let mut request = client.request(reqwest_method, target).body(body.to_vec());

    for (name, value) in &headers {
        if name == axum::http::header::HOST {
            continue;
        }
        request = request.header(name.as_str(), value.as_bytes());
    }

    let upstream = request
        .send()
        .await
        .map_err(|error| AppError::Internal(error.into()))?;
    let status = StatusCode::from_u16(upstream.status().as_u16())
        .map_err(|error| AppError::Internal(error.into()))?;
    let upstream_headers = upstream.headers().clone();
    let bytes = upstream
        .bytes()
        .await
        .map_err(|error| AppError::Internal(error.into()))?;
    let mut builder = Response::builder().status(status);

    for (name, value) in upstream_headers.iter() {
        if name == reqwest::header::CONTENT_LENGTH {
            continue;
        }
        builder = builder.header(name.as_str(), value.as_bytes());
    }

    builder
        .body(Body::from(bytes))
        .map_err(|error| AppError::Internal(error.into()))
}

fn build_proxy_url(proxy_url: &str, uri: &Uri) -> Result<String, AppError> {
    let mut target = url::Url::parse(proxy_url)
        .map_err(|_| AppError::BadRequest("dynamic profile proxy_url is invalid".to_string()))?;
    let base_path = target.path().trim_end_matches('/');
    let request_path = uri.path().trim_start_matches('/');
    let joined_path = if base_path.is_empty() {
        format!("/{request_path}")
    } else if request_path.is_empty() {
        base_path.to_string()
    } else {
        format!("{base_path}/{request_path}")
    };
    target.set_path(&joined_path);
    target.set_query(uri.query());
    Ok(target.to_string())
}

fn query_to_json(query: Option<&str>) -> Value {
    let mut result = Map::new();

    if let Some(query) = query {
        for (key, value) in form_urlencoded::parse(query.as_bytes()) {
            result.insert(key.into_owned(), Value::String(value.into_owned()));
        }
    }

    Value::Object(result)
}

fn headers_to_json(headers: &HeaderMap) -> Value {
    let mut result = Map::new();

    for (name, value) in headers {
        result.insert(
            name.as_str().to_string(),
            Value::String(value.to_str().unwrap_or("<non-utf8>").to_string()),
        );
    }

    Value::Object(result)
}

fn default_unknown_limit() -> u64 {
    50
}

fn default_scenario_name() -> String {
    "success".to_string()
}

fn default_profile_kind() -> ProfileKind {
    ProfileKind::Static
}

fn default_scenario_kind() -> ScenarioKind {
    ScenarioKind::Success
}

fn default_route_status() -> RouteStatus {
    RouteStatus::Active
}

fn default_status_code() -> i32 {
    200
}

fn default_response_headers() -> Value {
    json!({
        "content-type": "application/json"
    })
}

fn default_selection_rules() -> Value {
    json!({})
}

impl From<ConvertUnknownRequestPayload> for ConvertUnknownRequest {
    fn from(value: ConvertUnknownRequestPayload) -> Self {
        Self {
            name: value.name,
            tags: value.tags,
            scenario: CreateScenario {
                name: value.scenario.name,
                profile_kind: value.scenario.profile_kind,
                kind: value.scenario.kind,
                proxy_url: value.scenario.proxy_url,
                status_code: value.scenario.status_code,
                response_headers: value.scenario.response_headers,
                response_body: value.scenario.response_body,
                delay_ms: value.scenario.delay_ms,
                selection_rules: value.scenario.selection_rules,
            },
        }
    }
}

impl From<ConvertScenarioPayload> for CreateScenario {
    fn from(value: ConvertScenarioPayload) -> Self {
        Self {
            name: value.name,
            profile_kind: value.profile_kind,
            kind: value.kind,
            proxy_url: value.proxy_url,
            status_code: value.status_code,
            response_headers: value.response_headers,
            response_body: value.response_body,
            delay_ms: value.delay_ms,
            selection_rules: value.selection_rules,
        }
    }
}

impl From<RoutePayload> for UpsertRoute {
    fn from(value: RoutePayload) -> Self {
        Self {
            method: value.method,
            path_pattern: value.path_pattern,
            name: value.name,
            tags: value.tags,
            status: value.status,
            active_scenario_id: value.active_scenario_id,
        }
    }
}

impl From<UnknownRequest> for UnknownRequestResponse {
    fn from(request: UnknownRequest) -> Self {
        let body_base64 = request.body.as_ref().map(|body| STANDARD.encode(body));
        let body_text = request
            .body
            .as_ref()
            .and_then(|body| String::from_utf8(body.clone()).ok());

        Self {
            request,
            body_base64,
            body_text,
        }
    }
}
