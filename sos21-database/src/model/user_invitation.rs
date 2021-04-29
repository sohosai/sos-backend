use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::Type)]
#[sqlx(type_name = "user_invitation_role")]
#[sqlx(rename_all = "snake_case")]
pub enum UserInvitationRole {
    Administrator,
    CommitteeOperator,
    Committee,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct UserInvitation {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub author_id: String,
    pub email: String,
    pub role: UserInvitationRole,
}
