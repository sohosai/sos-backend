use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct FormAnswer {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub author_id: String,
    pub form_id: Uuid,
    pub project_id: Uuid,
    pub items: serde_json::Value,
}
