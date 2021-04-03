use crate::model::project::{ProjectAttribute, ProjectCategory};
use crate::model::user::UserId;

use chrono::{DateTime, Utc};
use sos21_domain::model::pending_project as entity;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PendingProjectId(pub Uuid);

impl PendingProjectId {
    pub fn from_entity(id: entity::PendingProjectId) -> Self {
        PendingProjectId(id.to_uuid())
    }

    pub fn into_entity(self) -> entity::PendingProjectId {
        entity::PendingProjectId::from_uuid(self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingProject {
    pub id: PendingProjectId,
    pub created_at: DateTime<Utc>,
    pub author_id: UserId,
    pub name: String,
    pub kana_name: String,
    pub group_name: String,
    pub kana_group_name: String,
    pub description: String,
    pub category: ProjectCategory,
    pub attributes: Vec<ProjectAttribute>,
}

impl PendingProject {
    pub fn from_entity(pending_project: entity::PendingProject) -> Self {
        PendingProject {
            id: PendingProjectId::from_entity(pending_project.id),
            created_at: pending_project.created_at.utc(),
            author_id: UserId::from_entity(pending_project.author_id),
            name: pending_project.name.into_string(),
            kana_name: pending_project.kana_name.into_string(),
            group_name: pending_project.group_name.into_string(),
            kana_group_name: pending_project.kana_group_name.into_string(),
            description: pending_project.description.into_string(),
            category: ProjectCategory::from_entity(pending_project.category),
            attributes: pending_project
                .attributes
                .attributes()
                .map(ProjectAttribute::from_entity)
                .collect(),
        }
    }
}
