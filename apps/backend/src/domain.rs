use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

pub fn is_valid_http_method(method: &str) -> bool {
    let method = method.trim();
    !method.is_empty()
        && method
            .bytes()
            .all(|byte| matches!(byte, b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'!' | b'#' | b'$' | b'%' | b'&' | b'\'' | b'*' | b'+' | b'-' | b'.' | b'^' | b'_' | b'`' | b'|' | b'~'))
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RouteStatus {
    Active,
    Disabled,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ScenarioKind {
    Success,
    Error,
    Timeout,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProfileKind {
    Static,
    Dynamic,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum UnknownRequestStatus {
    New,
    Ignored,
    Converted,
}

#[derive(Debug, Clone)]
pub struct CapturedRequest {
    pub method: String,
    pub path: String,
    pub query: Value,
    pub headers: Value,
    pub body: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: Uuid,
    pub name: String,
    #[serde(default)]
    pub key: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct CreateProject {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnknownRequest {
    pub id: Uuid,
    pub project_id: Uuid,
    pub method: String,
    pub path: String,
    pub query: Value,
    pub headers: Value,
    #[serde(skip_serializing, default)]
    pub body: Option<Vec<u8>>,
    pub first_seen_at: DateTime<Utc>,
    pub last_seen_at: DateTime<Utc>,
    pub count: i64,
    pub status: UnknownRequestStatus,
    pub converted_route_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockRoute {
    pub id: Uuid,
    pub project_id: Uuid,
    pub method: String,
    pub path_pattern: String,
    pub name: String,
    pub tags: Vec<String>,
    pub status: RouteStatus,
    pub active_scenario_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseScenario {
    pub id: Uuid,
    pub route_id: Uuid,
    pub name: String,
    pub profile_kind: ProfileKind,
    pub kind: ScenarioKind,
    pub proxy_url: Option<String>,
    pub status_code: i32,
    pub response_headers: Value,
    pub response_body: Option<String>,
    pub delay_ms: i32,
    pub selection_rules: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveMockResponse {
    pub route: MockRoute,
    pub scenario: ResponseScenario,
}

#[derive(Debug, Clone)]
pub struct CreateScenario {
    pub name: String,
    pub profile_kind: ProfileKind,
    pub kind: ScenarioKind,
    pub proxy_url: Option<String>,
    pub status_code: i32,
    pub response_headers: Value,
    pub response_body: Option<String>,
    pub delay_ms: i32,
    pub selection_rules: Value,
}

#[derive(Debug, Clone)]
pub struct ConvertUnknownRequest {
    pub name: Option<String>,
    pub tags: Vec<String>,
    pub scenario: CreateScenario,
}

#[derive(Debug, Clone)]
pub struct UpsertRoute {
    pub method: String,
    pub path_pattern: String,
    pub name: String,
    pub tags: Vec<String>,
    pub status: RouteStatus,
    pub active_scenario_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvertedUnknownRequest {
    pub route: MockRoute,
    pub scenario: ResponseScenario,
    pub unknown_request: UnknownRequest,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectAsset {
    pub bucket: String,
    pub object_key: String,
    pub content_type: Option<String>,
    pub size_bytes: i64,
}
