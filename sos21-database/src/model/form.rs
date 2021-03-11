use crate::model::project::{ProjectAttributes, ProjectCategory};

use chrono::{DateTime, Utc};
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
    pub items: serde_json::Value,
}

#[derive(Debug, Clone)]
pub struct FormData {
    pub form: Form,
    pub include_ids: Vec<Uuid>,
    pub exclude_ids: Vec<Uuid>,
    pub query: Vec<FormProjectQueryConjunction>,
}

#[derive(Debug, Clone)]
pub struct FormProjectQueryConjunction {
    pub category: Option<ProjectCategory>,
    pub attributes: ProjectAttributes,
}
