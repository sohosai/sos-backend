use crate::handler::model::user::{UserId, UserKanaName, UserName};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sos21_use_case::model::project as use_case;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ProjectId(pub Uuid);

impl ProjectId {
    pub fn from_use_case(id: use_case::ProjectId) -> ProjectId {
        ProjectId(id.0)
    }

    pub fn into_use_case(self) -> use_case::ProjectId {
        use_case::ProjectId(self.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectCategory {
    General,
    Stage,
    Cooking,
    Food,
}

impl ProjectCategory {
    pub fn from_use_case(category: use_case::ProjectCategory) -> ProjectCategory {
        match category {
            use_case::ProjectCategory::General => ProjectCategory::General,
            use_case::ProjectCategory::Stage => ProjectCategory::Stage,
            use_case::ProjectCategory::Cooking => ProjectCategory::Cooking,
            use_case::ProjectCategory::Food => ProjectCategory::Food,
        }
    }

    pub fn into_use_case(self) -> use_case::ProjectCategory {
        match self {
            ProjectCategory::General => use_case::ProjectCategory::General,
            ProjectCategory::Stage => use_case::ProjectCategory::Stage,
            ProjectCategory::Cooking => use_case::ProjectCategory::Cooking,
            ProjectCategory::Food => use_case::ProjectCategory::Food,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectAttribute {
    Academic,
    Artistic,
    Committee,
    Outdoor,
}

impl ProjectAttribute {
    pub fn from_use_case(attribute: use_case::ProjectAttribute) -> ProjectAttribute {
        match attribute {
            use_case::ProjectAttribute::Academic => ProjectAttribute::Academic,
            use_case::ProjectAttribute::Artistic => ProjectAttribute::Artistic,
            use_case::ProjectAttribute::Committee => ProjectAttribute::Committee,
            use_case::ProjectAttribute::Outdoor => ProjectAttribute::Outdoor,
        }
    }

    pub fn into_use_case(self) -> use_case::ProjectAttribute {
        match self {
            ProjectAttribute::Academic => use_case::ProjectAttribute::Academic,
            ProjectAttribute::Artistic => use_case::ProjectAttribute::Artistic,
            ProjectAttribute::Committee => use_case::ProjectAttribute::Committee,
            ProjectAttribute::Outdoor => use_case::ProjectAttribute::Outdoor,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: ProjectId,
    pub code: String,
    pub created_at: DateTime<Utc>,
    pub owner_id: UserId,
    pub owner_name: UserName,
    pub owner_kana_name: UserKanaName,
    pub name: String,
    pub kana_name: String,
    pub group_name: String,
    pub kana_group_name: String,
    pub description: String,
    pub category: ProjectCategory,
    pub attributes: Vec<ProjectAttribute>,
}

impl Project {
    pub fn from_use_case(project: use_case::Project) -> Project {
        Project {
            id: ProjectId::from_use_case(project.id),
            code: project.code,
            created_at: project.created_at,
            owner_id: UserId::from_use_case(project.owner_id),
            owner_name: UserName::from_use_case(project.owner_name),
            owner_kana_name: UserKanaName::from_use_case(project.owner_kana_name),
            name: project.name,
            kana_name: project.kana_name,
            group_name: project.group_name,
            kana_group_name: project.kana_group_name,
            description: project.description,
            category: ProjectCategory::from_use_case(project.category),
            attributes: project
                .attributes
                .into_iter()
                .map(ProjectAttribute::from_use_case)
                .collect(),
        }
    }
}
