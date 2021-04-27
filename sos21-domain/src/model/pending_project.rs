use crate::model::date_time::DateTime;
use crate::model::project::{
    ProjectAttributes, ProjectCategory, ProjectDescription, ProjectGroupName, ProjectKanaGroupName,
    ProjectKanaName, ProjectName,
};
use crate::model::user::{User, UserId};

use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PendingProjectId(Uuid);

impl PendingProjectId {
    pub fn from_uuid(uuid: Uuid) -> PendingProjectId {
        PendingProjectId(uuid)
    }

    pub fn to_uuid(&self) -> Uuid {
        self.0
    }
}

#[derive(Debug, Clone)]
pub struct PendingProject {
    pub id: PendingProjectId,
    pub created_at: DateTime,
    pub author_id: UserId,
    pub name: ProjectName,
    pub kana_name: ProjectKanaName,
    pub group_name: ProjectGroupName,
    pub kana_group_name: ProjectKanaGroupName,
    pub description: ProjectDescription,
    pub category: ProjectCategory,
    pub attributes: ProjectAttributes,
}

impl PendingProject {
    pub fn is_visible_to(&self, _user: &User) -> bool {
        true
    }
}
