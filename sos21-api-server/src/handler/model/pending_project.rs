use crate::handler::model::date_time::DateTime;
use crate::handler::model::project::{ProjectAttribute, ProjectCategory};
use crate::handler::model::user::UserId;

use serde::{Deserialize, Serialize};
use sos21_use_case::model::pending_project as use_case;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PendingProjectId(pub Uuid);

impl PendingProjectId {
    pub fn from_use_case(id: use_case::PendingProjectId) -> PendingProjectId {
        PendingProjectId(id.0)
    }

    pub fn into_use_case(self) -> use_case::PendingProjectId {
        use_case::PendingProjectId(self.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingProject {
    pub id: PendingProjectId,
    pub created_at: DateTime,
    pub updated_at: DateTime,
    pub owner_id: UserId,
    pub name: String,
    pub kana_name: String,
    pub group_name: String,
    pub kana_group_name: String,
    pub description: String,
    pub category: ProjectCategory,
    pub attributes: Vec<ProjectAttribute>,
}

impl PendingProject {
    pub fn from_use_case(pending_project: use_case::PendingProject) -> PendingProject {
        PendingProject {
            id: PendingProjectId::from_use_case(pending_project.id),
            created_at: DateTime::from_use_case(pending_project.created_at),
            updated_at: DateTime::from_use_case(pending_project.updated_at),
            owner_id: UserId::from_use_case(pending_project.owner_id),
            name: pending_project.name,
            kana_name: pending_project.kana_name,
            group_name: pending_project.group_name,
            kana_group_name: pending_project.kana_group_name,
            description: pending_project.description,
            category: ProjectCategory::from_use_case(pending_project.category),
            attributes: pending_project
                .attributes
                .into_iter()
                .map(ProjectAttribute::from_use_case)
                .collect(),
        }
    }
}
