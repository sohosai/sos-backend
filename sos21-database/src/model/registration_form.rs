use crate::model::project::{ProjectAttributes, ProjectCategory};

use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct RegistrationForm {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub author_id: String,
    pub name: String,
    pub description: String,
    pub items: serde_json::Value,
}

#[derive(Debug, Clone)]
pub struct RegistrationFormData {
    pub registration_form: RegistrationForm,
    pub query: Vec<RegistrationFormProjectQueryConjunction>,
}

#[derive(Debug, Clone)]
pub struct RegistrationFormProjectQueryConjunction {
    pub category: Option<ProjectCategory>,
    pub attributes: ProjectAttributes,
}
