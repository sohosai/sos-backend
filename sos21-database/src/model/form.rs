use chrono::{DateTime, Utc};
use sqlx::types::BitVec;
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Form {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub author_id: String,
    pub name: String,
    pub description: String,
    pub starts_at: DateTime<Utc>,
    pub ends_at: DateTime<Utc>,
    pub items: Vec<u8>,
    pub condition: Vec<u8>,
    // TODO: u64-based bit vector?
    pub unspecified_query: BitVec,
    pub general_query: BitVec,
    pub stage_query: BitVec,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct FormConditionInclude {
    pub project_id: Uuid,
    pub form_id: Uuid,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct FormConditionExclude {
    pub project_id: Uuid,
    pub form_id: Uuid,
}
