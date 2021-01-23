use chrono::{DateTime, Utc};

#[derive(Debug, Clone, sqlx::Type)]
#[sqlx(type_name = "USER_ROLE")]
#[sqlx(rename_all = "lowercase")]
pub enum UserRole {
    Administrator,
    Committee,
    General,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct User {
    pub id: String,
    pub created_at: DateTime<Utc>,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub role: UserRole,
}
