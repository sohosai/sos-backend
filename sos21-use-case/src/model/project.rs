use crate::model::user::{UserId, UserName};

use chrono::{DateTime, Utc};
use sos21_domain_model::project as entity;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProjectId(pub Uuid);

impl ProjectId {
    pub fn from_entity(id: entity::ProjectId) -> ProjectId {
        ProjectId(id.0)
    }

    pub fn into_entity(self) -> entity::ProjectId {
        entity::ProjectId(self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ProjectCategory {
    General,
    Stage,
}

impl ProjectCategory {
    pub fn from_entity(category: entity::ProjectCategory) -> ProjectCategory {
        match category {
            entity::ProjectCategory::General => ProjectCategory::General,
            entity::ProjectCategory::Stage => ProjectCategory::Stage,
        }
    }

    pub fn into_entity(self) -> entity::ProjectCategory {
        match self {
            ProjectCategory::General => entity::ProjectCategory::General,
            ProjectCategory::Stage => entity::ProjectCategory::Stage,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ProjectAttribute {
    Academic,
    Artistic,
    Committee,
}

impl ProjectAttribute {
    pub fn from_entity(attribute: entity::ProjectAttribute) -> ProjectAttribute {
        match attribute {
            entity::ProjectAttribute::Academic => ProjectAttribute::Academic,
            entity::ProjectAttribute::Artistic => ProjectAttribute::Artistic,
            entity::ProjectAttribute::Committee => ProjectAttribute::Committee,
        }
    }

    pub fn into_entity(self) -> entity::ProjectAttribute {
        match self {
            ProjectAttribute::Academic => entity::ProjectAttribute::Academic,
            ProjectAttribute::Artistic => entity::ProjectAttribute::Artistic,
            ProjectAttribute::Committee => entity::ProjectAttribute::Committee,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Project {
    pub id: ProjectId,
    pub created_at: DateTime<Utc>,
    pub display_id: String,
    pub owner_id: UserId,
    pub owner_name: UserName,
    pub name: String,
    pub kana_name: String,
    pub group_name: String,
    pub kana_group_name: String,
    pub description: String,
    pub category: ProjectCategory,
    pub attributes: Vec<ProjectAttribute>,
}

impl Project {
    pub fn from_entity(
        project: entity::Project,
        owner_name: sos21_domain_model::user::UserName,
    ) -> Project {
        Project {
            id: ProjectId::from_entity(project.id),
            created_at: project.created_at,
            display_id: project.display_id.into_string(),
            owner_id: UserId::from_entity(project.owner_id),
            owner_name: UserName::from_entity(owner_name),
            name: project.name.into_string(),
            kana_name: project.kana_name.into_string(),
            group_name: project.group_name.into_string(),
            kana_group_name: project.kana_group_name.into_string(),
            description: project.description.into_string(),
            category: ProjectCategory::from_entity(project.category),
            attributes: project
                .attributes
                .attributes()
                .map(ProjectAttribute::from_entity)
                .collect(),
        }
    }
}
