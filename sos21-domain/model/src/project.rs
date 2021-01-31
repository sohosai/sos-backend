use crate::permissions::Permissions;
use crate::user::{User, UserId};

use chrono::{DateTime, Utc};
use uuid::Uuid;

pub mod attribute;
pub mod category;
pub mod description;
pub mod display_id;
pub mod name;
pub use attribute::{ProjectAttribute, ProjectAttributes};
pub use category::ProjectCategory;
pub use description::ProjectDescription;
pub use display_id::ProjectDisplayId;
pub use name::{ProjectGroupName, ProjectKanaGroupName, ProjectKanaName, ProjectName};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ProjectId(pub Uuid);

impl ProjectId {
    pub fn from_uuid(uuid: Uuid) -> ProjectId {
        ProjectId(uuid)
    }

    pub fn to_uuid(&self) -> Uuid {
        self.0
    }
}

#[derive(Debug, Clone)]
pub struct Project {
    pub id: ProjectId,
    pub created_at: DateTime<Utc>,
    pub display_id: ProjectDisplayId,
    pub owner_id: UserId,
    pub name: ProjectName,
    pub kana_name: ProjectKanaName,
    pub group_name: ProjectGroupName,
    pub kana_group_name: ProjectKanaGroupName,
    pub description: ProjectDescription,
    pub category: ProjectCategory,
    pub attributes: ProjectAttributes,
}

impl Project {
    pub fn is_visible_to(&self, user: &User) -> bool {
        if self.owner_id == user.id {
            return true;
        }

        user.permissions().contains(Permissions::READ_ALL_PROJECTS)
    }

    pub fn set_name(&mut self, name: ProjectName) {
        self.name = name;
    }

    pub fn set_kana_name(&mut self, kana_name: ProjectKanaName) {
        self.kana_name = kana_name;
    }

    pub fn set_group_name(&mut self, group_name: ProjectGroupName) {
        self.group_name = group_name;
    }

    pub fn set_kana_group_name(&mut self, kana_group_name: ProjectKanaGroupName) {
        self.kana_group_name = kana_group_name;
    }

    pub fn set_description(&mut self, description: ProjectDescription) {
        self.description = description;
    }

    pub fn set_category(&mut self, category: ProjectCategory) {
        self.category = category;
    }

    pub fn set_attributes(&mut self, attributes: ProjectAttributes) {
        self.attributes = attributes;
    }
}

#[cfg(test)]
mod tests {
    use sos21_domain_test::model as test_model;

    #[test]
    fn test_visibility_general_owner() {
        let user = test_model::new_general_user();
        let project = test_model::new_general_project(user.id.clone());
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
