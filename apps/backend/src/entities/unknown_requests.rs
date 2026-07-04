use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(
    rs_type = "String",
    db_type = "Enum",
    enum_name = "unknown_request_status"
)]
pub enum UnknownRequestStatus {
    #[sea_orm(string_value = "new")]
    New,
    #[sea_orm(string_value = "ignored")]
    Ignored,
    #[sea_orm(string_value = "converted")]
    Converted,
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "unknown_requests")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub project_id: Uuid,
    pub method: String,
    pub path: String,
    pub query: Value,
    pub headers: Value,
    pub body: Option<Vec<u8>>,
    pub first_seen_at: DateTime<Utc>,
    pub last_seen_at: DateTime<Utc>,
    pub count: i64,
    pub status: UnknownRequestStatus,
    pub converted_route_id: Option<Uuid>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
