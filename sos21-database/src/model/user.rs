use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::Type)]
#[sqlx(type_name = "user_role")]
#[sqlx(rename_all = "snake_case")]
pub enum UserRole {
    Administrator,
    CommitteeOperator,
    Committee,
    General,
}

#[derive(Debug, Clone, sqlx::Type)]
#[sqlx(type_name = "user_category")]
#[sqlx(rename_all = "snake_case")]
pub enum UserCategory {
    UndergraduateStudent,
    GraduateStudent,
    AcademicStaff,
}

#[derive(Debug, Clone, sqlx::Type)]
#[sqlx(type_name = "user_assignment")]
#[sqlx(rename_all = "snake_case")]
pub enum UserAssignment {
    ProjectOwner,
    ProjectSubowner,
    PendingProjectOwner,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct User {
    pub id: String,
    pub created_at: DateTime<Utc>,
    pub first_name: String,
    pub kana_first_name: String,
    pub last_name: String,
    pub kana_last_name: String,
    pub phone_number: String,
    pub email: String,
    pub role: UserRole,
    pub category: UserCategory,
    pub assignment: Option<UserAssignment>,
    pub assignment_owner_project_id: Option<Uuid>,
    pub assignment_subowner_project_id: Option<Uuid>,
    pub assignment_owner_pending_project_id: Option<Uuid>,
}
