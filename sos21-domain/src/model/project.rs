use crate::context::ProjectRepository;
use crate::model::date_time::DateTime;
use crate::model::permissions::Permissions;
use crate::model::user::{User, UserId};

use thiserror::Error;
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
    pub created_at: DateTime,
    // TODO: encupsulate to ensure availability
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

#[derive(Debug, Error, Clone)]
#[error("the project display id is not available")]
pub struct ProjectDisplayIdNotAvailableError {
    _priv: (),
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

    pub async fn set_display_id<C>(
        &mut self,
        ctx: C,
        display_id: ProjectDisplayId,
    ) -> anyhow::Result<Result<(), ProjectDisplayIdNotAvailableError>>
    where
        C: ProjectRepository,
    {
        if !display_id.is_available(ctx).await? {
            return Ok(Err(ProjectDisplayIdNotAvailableError { _priv: () }));
        }

        self.display_id = display_id;
        Ok(Ok(()))
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

    #[tokio::test]
    async fn test_set_display_id_unavailable() {
        use sos21_domain_test as test;

        let user = test::model::new_general_user();
        let mut project1 = test::model::new_general_project(user.id.clone());
        let project2 = test::model::new_general_project(user.id.clone());
        let app = test::build_mock_app()
            .users(vec![user.clone()])
            .projects(vec![project1.clone(), project2.clone()])
            .build();

        assert!(matches!(
            project1
                .set_display_id(app, project2.display_id)
                .await
                .unwrap(),
            Err(_)
        ));
    }

    #[tokio::test]
    async fn test_set_display_id_available() {
        use sos21_domain_test as test;

        let user = test::model::new_general_user();
        let mut project = test::model::new_general_project(user.id.clone());
        let display_id = test::model::new_project_display_id();
        let app = test::build_mock_app()
            .users(vec![user.clone()])
            .projects(vec![project.clone()])
            .build();

        assert!(project
            .set_display_id(app, display_id.clone())
            .await
            .unwrap()
            .is_ok());
        assert_eq!(project.display_id, display_id);
    }
}
