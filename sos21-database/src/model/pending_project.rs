use crate::model::project::{ProjectAttributes, ProjectCategory};
use crate::model::user::User;

use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct PendingProject {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub name: String,
    pub kana_name: String,
    pub group_name: String,
    pub kana_group_name: String,
    pub description: String,
    pub category: ProjectCategory,
    pub attributes: ProjectAttributes,
}

#[derive(Debug, Clone)]
pub struct PendingProjectWithOwner {
    pub pending_project: PendingProject,
    pub owner: User,
}
