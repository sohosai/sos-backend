use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct RegistrationFormAnswer {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub author_id: String,
    pub registration_form_id: Uuid,
    pub project_id: Option<Uuid>,
    pub pending_project_id: Option<Uuid>,
    pub items: serde_json::Value,
}
