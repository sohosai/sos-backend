use crate::model::user::{UserId, UserKanaName, UserName};

use chrono::{DateTime, Utc};
use sos21_domain::model::project as entity;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProjectId(pub Uuid);

impl ProjectId {
    pub fn from_entity(id: entity::ProjectId) -> ProjectId {
        ProjectId(id.to_uuid())
    }

    pub fn into_entity(self) -> entity::ProjectId {
        entity::ProjectId::from_uuid(self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ProjectCategory {
    General,
    Stage,
    Cooking,
    Food,
}

impl ProjectCategory {
    pub fn from_entity(category: entity::ProjectCategory) -> ProjectCategory {
        match category {
            entity::ProjectCategory::General => ProjectCategory::General,
            entity::ProjectCategory::Stage => ProjectCategory::Stage,
            entity::ProjectCategory::Cooking => ProjectCategory::Cooking,
            entity::ProjectCategory::Food => ProjectCategory::Food,
        }
    }

    pub fn into_entity(self) -> entity::ProjectCategory {
        match self {
            ProjectCategory::General => entity::ProjectCategory::General,
            ProjectCategory::Stage => entity::ProjectCategory::Stage,
            ProjectCategory::Cooking => entity::ProjectCategory::Cooking,
            ProjectCategory::Food => entity::ProjectCategory::Food,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ProjectAttribute {
    Academic,
    Artistic,
    Committee,
    Outdoor,
}

impl ProjectAttribute {
    pub fn from_entity(attribute: entity::ProjectAttribute) -> ProjectAttribute {
        match attribute {
            entity::ProjectAttribute::Academic => ProjectAttribute::Academic,
            entity::ProjectAttribute::Artistic => ProjectAttribute::Artistic,
            entity::ProjectAttribute::Committee => ProjectAttribute::Committee,
            entity::ProjectAttribute::Outdoor => ProjectAttribute::Outdoor,
        }
    }

    pub fn into_entity(self) -> entity::ProjectAttribute {
        match self {
            ProjectAttribute::Academic => entity::ProjectAttribute::Academic,
            ProjectAttribute::Artistic => entity::ProjectAttribute::Artistic,
            ProjectAttribute::Committee => entity::ProjectAttribute::Committee,
            ProjectAttribute::Outdoor => entity::ProjectAttribute::Outdoor,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Project {
    pub id: ProjectId,
    pub code: String,
    pub created_at: DateTime<Utc>,
    pub owner_id: UserId,
    pub owner_name: UserName,
    pub owner_kana_name: UserKanaName,
    pub subowner_id: UserId,
    pub subowner_name: UserName,
    pub subowner_kana_name: UserKanaName,
    pub name: String,
    pub kana_name: String,
    pub group_name: String,
    pub kana_group_name: String,
    pub description: String,
    pub category: ProjectCategory,
    pub attributes: Vec<ProjectAttribute>,
}

#[derive(Debug, Clone)]
pub struct ProjectFromEntityInput {
    pub project: entity::Project,
    pub owner_name: sos21_domain::model::user::UserName,
    pub owner_kana_name: sos21_domain::model::user::UserKanaName,
    pub subowner_name: sos21_domain::model::user::UserName,
    pub subowner_kana_name: sos21_domain::model::user::UserKanaName,
}

impl Project {
    pub fn from_entity(input: ProjectFromEntityInput) -> Self {
        let ProjectFromEntityInput {
            project,
            owner_name,
            owner_kana_name,
            subowner_name,
            subowner_kana_name,
        } = input;

        Project {
            id: ProjectId::from_entity(project.id()),
            code: project.code().to_string(),
            created_at: project.created_at().utc(),
            owner_id: UserId::from_entity(project.owner_id().clone()),
            owner_name: UserName::from_entity(owner_name),
            owner_kana_name: UserKanaName::from_entity(owner_kana_name),
            subowner_id: UserId::from_entity(project.subowner_id().clone()),
            subowner_name: UserName::from_entity(subowner_name),
            subowner_kana_name: UserKanaName::from_entity(subowner_kana_name),
            name: project.name().clone().into_string(),
            kana_name: project.kana_name().clone().into_string(),
            group_name: project.group_name().clone().into_string(),
            kana_group_name: project.kana_group_name().clone().into_string(),
            description: project.description().clone().into_string(),
            category: ProjectCategory::from_entity(project.category()),
            attributes: project
                .attributes()
                .attributes()
                .map(ProjectAttribute::from_entity)
                .collect(),
        }
    }
}
