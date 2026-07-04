use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "scenario_kind")]
pub enum ScenarioKind {
    #[sea_orm(string_value = "success")]
    Success,
    #[sea_orm(string_value = "error")]
    Error,
    #[sea_orm(string_value = "timeout")]
    Timeout,
    #[sea_orm(string_value = "custom")]
    Custom,
}

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "profile_kind")]
pub enum ProfileKind {
    #[sea_orm(string_value = "static")]
    Static,
    #[sea_orm(string_value = "dynamic")]
    Dynamic,
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "response_scenarios")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub route_id: Uuid,
    pub name: String,
    pub profile_kind: ProfileKind,
    pub kind: ScenarioKind,
    pub proxy_url: Option<String>,
    pub proxy_url_mode: String,
    pub status_code: i32,
    pub response_headers: Value,
    pub response_body: Option<String>,
    pub delay_ms: i32,
    pub selection_rules: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
