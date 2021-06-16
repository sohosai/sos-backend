use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::Type)]
#[sqlx(type_name = "file_sharing_scope")]
#[sqlx(rename_all = "snake_case")]
pub enum FileSharingScope {
    Project,
    ProjectQuery,
    FormAnswer,
    RegistrationFormAnswer,
    Committee,
    CommitteeOperator,
    Public,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct FileSharing {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub file_id: Uuid,
    pub is_revoked: bool,
    pub expires_at: Option<DateTime<Utc>>,
    pub scope: FileSharingScope,
    pub project_id: Option<Uuid>,
    pub project_query: Option<serde_json::Value>,
    pub form_answer_project_id: Option<Uuid>,
    pub form_answer_form_id: Option<Uuid>,
    pub registration_form_answer_project_id: Option<Uuid>,
    pub registration_form_answer_pending_project_id: Option<Uuid>,
    pub registration_form_answer_registration_form_id: Option<Uuid>,
}
