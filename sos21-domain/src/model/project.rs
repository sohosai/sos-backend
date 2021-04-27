use crate::model::date_time::DateTime;
use crate::model::permissions::Permissions;
use crate::model::user::{User, UserId};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

pub mod attribute;
pub mod category;
pub mod code;
pub mod description;
pub mod index;
pub mod name;
pub use attribute::{ProjectAttribute, ProjectAttributes};
pub use category::ProjectCategory;
pub use code::{ProjectCode, ProjectKind};
pub use description::ProjectDescription;
pub use index::ProjectIndex;
pub use name::{ProjectGroupName, ProjectKanaGroupName, ProjectKanaName, ProjectName};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ProjectId(Uuid);

impl ProjectId {
    pub fn from_uuid(uuid: Uuid) -> ProjectId {
        ProjectId(uuid)
    }

    pub fn to_uuid(&self) -> Uuid {
        self.0
    }
}

#[derive(Debug, Clone)]
pub struct ProjectContent {
    pub id: ProjectId,
    pub index: ProjectIndex,
    pub created_at: DateTime,
    pub owner_id: UserId,
    pub subowner_id: UserId,
    pub name: ProjectName,
    pub kana_name: ProjectKanaName,
    pub group_name: ProjectGroupName,
    pub kana_group_name: ProjectKanaGroupName,
    pub description: ProjectDescription,
    pub category: ProjectCategory,
    pub attributes: ProjectAttributes,
}

#[derive(Debug, Clone)]
pub struct Project(ProjectContent);

#[derive(Debug, Clone, Error)]
#[error("invalid project with same owner and subowner")]
pub struct SameOwnerSubownerError {
    _priv: (),
}

impl Project {
    pub fn from_content(content: ProjectContent) -> Result<Self, SameOwnerSubownerError> {
        if content.owner_id == content.subowner_id {
            return Err(SameOwnerSubownerError { _priv: () });
        }

        Ok(Project(content))
    }

    pub fn into_content(self) -> ProjectContent {
        self.0
    }

    pub fn id(&self) -> ProjectId {
        self.0.id
    }

    pub fn index(&self) -> ProjectIndex {
        self.0.index
    }

    pub fn created_at(&self) -> DateTime {
        self.0.created_at
    }

    pub fn name(&self) -> &ProjectName {
        &self.0.name
    }

    pub fn kana_name(&self) -> &ProjectKanaName {
        &self.0.kana_name
    }

    pub fn group_name(&self) -> &ProjectGroupName {
        &self.0.group_name
    }

    pub fn kana_group_name(&self) -> &ProjectKanaGroupName {
        &self.0.kana_group_name
    }

    pub fn description(&self) -> &ProjectDescription {
        &self.0.description
    }

    pub fn category(&self) -> ProjectCategory {
        self.0.category
    }

    pub fn attributes(&self) -> &ProjectAttributes {
        &self.0.attributes
    }

    pub fn owner_id(&self) -> &UserId {
        &self.0.owner_id
    }

    pub fn subowner_id(&self) -> &UserId {
        &self.0.subowner_id
    }

    pub fn is_visible_to(&self, user: &User) -> bool {
        if self.0.owner_id == user.id || self.0.subowner_id == user.id {
            return true;
        }

        user.permissions().contains(Permissions::READ_ALL_PROJECTS)
    }

    pub fn kind(&self) -> ProjectKind {
        ProjectKind {
            is_cooking: self.0.category == ProjectCategory::Cooking,
            is_outdoor: self.0.attributes.contains(ProjectAttribute::Outdoor),
        }
    }

    pub fn code(&self) -> ProjectCode {
        ProjectCode {
            kind: self.kind(),
            index: self.0.index,
        }
    }

    pub fn set_name(&mut self, name: ProjectName) {
        self.0.name = name;
    }

    pub fn set_kana_name(&mut self, kana_name: ProjectKanaName) {
        self.0.kana_name = kana_name;
    }

    pub fn set_group_name(&mut self, group_name: ProjectGroupName) {
        self.0.group_name = group_name;
    }

    pub fn set_kana_group_name(&mut self, kana_group_name: ProjectKanaGroupName) {
        self.0.kana_group_name = kana_group_name;
    }

    pub fn set_description(&mut self, description: ProjectDescription) {
        self.0.description = description;
    }

    pub fn set_category(&mut self, category: ProjectCategory) {
        self.0.category = category;
    }

    pub fn set_attributes(&mut self, attributes: ProjectAttributes) {
        self.0.attributes = attributes;
    }
}

#[cfg(test)]
mod tests {
    use crate::test::model as test_model;

    #[test]
    fn test_visibility_general_owner() {
        let user = test_model::new_general_user();
        let project = test_model::new_general_project(user.id.clone());
        assert!(project.is_visible_to(&user));
    }

    #[test]
    fn test_visibility_general_subowner() {
        let user = test_model::new_general_user();
        let project = test_model::new_general_project_with_subowner(
            test_model::new_user_id(),
            user.id.clone(),
        );
        assert!(project.is_visible_to(&user));
    }

    #[test]
    fn test_visibility_general_other() {
        let user = test_model::new_general_user();
        let other = test_model::new_general_user();
        let project = test_model::new_general_project(other.id.clone());
        assert!(!project.is_visible_to(&user));
    }

    #[test]
    fn test_visibility_committee_other() {
        let user = test_model::new_committee_user();
        let other = test_model::new_general_user();
        let project = test_model::new_general_project(other.id.clone());
        assert!(project.is_visible_to(&user));
    }

    #[test]
    fn test_visibility_operator_other() {
        let user = test_model::new_operator_user();
        let other = test_model::new_general_user();
        let project = test_model::new_general_project(other.id.clone());
        assert!(project.is_visible_to(&user));
    }
}
