use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct File {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub author_id: String,
    pub object_id: Uuid,
    pub name: Option<String>,
    pub type_: String,
    pub size: i64,
}
