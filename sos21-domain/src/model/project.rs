use std::convert::TryInto;

use crate::context::{
    ProjectRepository, RegistrationFormAnswerRepository, RegistrationFormRepository,
};
use crate::model::date_time::DateTime;
use crate::model::pending_project::PendingProject;
use crate::model::permissions::Permissions;
use crate::model::user::{User, UserId};

use anyhow::Context;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NewProjectErrorKind {
    TooManyProjects,
    SameOwnerSubowner,
    NotAnsweredRegistrationForm,
}

#[derive(Debug, Clone, Error)]
#[error("failed to create a project")]
pub struct NewProjectError {
    kind: NewProjectErrorKind,
}

impl NewProjectError {
    pub fn kind(&self) -> NewProjectErrorKind {
        self.kind
    }

    fn from_count_integer_error(_err: std::num::TryFromIntError) -> Self {
        NewProjectError {
            kind: NewProjectErrorKind::TooManyProjects,
        }
    }

    fn from_index_error(_err: index::FromU16Error) -> Self {
        NewProjectError {
            kind: NewProjectErrorKind::TooManyProjects,
        }
    }

    fn from_content_error(_err: SameOwnerSubownerError) -> Self {
        NewProjectError {
            kind: NewProjectErrorKind::SameOwnerSubowner,
        }
    }
}

impl Project {
    pub async fn new<C>(
        ctx: C,
        pending_project: PendingProject,
        subowner: &User,
    ) -> anyhow::Result<Result<Self, NewProjectError>>
    where
        C: ProjectRepository + RegistrationFormRepository + RegistrationFormAnswerRepository,
    {
        let forms_count = ctx
            .count_registration_forms_by_pending_project(pending_project.id)
            .await
            .context("Failed to count registration forms")?;
        let answers_count = ctx
            .count_registration_form_answers_by_pending_project(pending_project.id)
            .await
            .context("Failed to count registration form answers")?;
        if forms_count != answers_count {
            return Ok(Err(NewProjectError {
                kind: NewProjectErrorKind::NotAnsweredRegistrationForm,
            }));
        }

        let projects_count = ctx
            .count_projects()
            .await
            .context("Failed to count projects")?;
        let projects_count = match projects_count.try_into() {
            Ok(count) => count,
            Err(err) => return Ok(Err(NewProjectError::from_count_integer_error(err))),
        };
        let index = match ProjectIndex::from_u16(projects_count) {
            Ok(index) => index,
            Err(err) => return Ok(Err(NewProjectError::from_index_error(err))),
        };

        Ok(Project::from_content(ProjectContent {
            id: ProjectId::from_uuid(Uuid::new_v4()),
            created_at: DateTime::now(),
            index,
            owner_id: pending_project.author_id,
            subowner_id: subowner.id.clone(),
            name: pending_project.name,
            kana_name: pending_project.kana_name,
            group_name: pending_project.group_name,
            kana_group_name: pending_project.kana_group_name,
            description: pending_project.description,
            category: pending_project.category,
            attributes: pending_project.attributes,
        })
        .map_err(NewProjectError::from_content_error))
    }

    /// Restore `Project` from `ProjectContent`.
    ///
    /// This is intended to be used when the data is taken out of the implementation by [`Project::into_content`]
    /// for persistence, internal serialization, etc.
    /// Use [`Project::new`] to create a project.
    pub fn from_content(content: ProjectContent) -> Result<Self, SameOwnerSubownerError> {
        if content.owner_id == content.subowner_id {
            return Err(SameOwnerSubownerError { _priv: () });
        }

        Ok(Project(content))
    }

    /// Convert `Project` into `ProjectContent`.
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
    use super::{NewProjectErrorKind, Project};
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

    #[tokio::test]
    async fn test_new_ok() {
        let owner = test_model::new_general_user();
        let subowner = test_model::new_general_user();
        let pending_project = test_model::new_general_pending_project(owner.id.clone());

        let app = crate::test::build_mock_app()
            .users(vec![owner.clone(), subowner.clone()])
            .pending_projects(vec![pending_project.clone()])
            .build();

        let project = Project::new(&app, pending_project, &subowner)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(project.owner_id(), &owner.id);
        assert_eq!(project.subowner_id(), &subowner.id);
    }

    #[tokio::test]
    async fn test_new_not_answered() {
        let owner = test_model::new_general_user();
        let subowner = test_model::new_general_user();
        let pending_project = test_model::new_general_pending_project(owner.id.clone());

        let operator = test_model::new_general_user();
        let registration_form = test_model::new_registration_form(operator.id.clone());

        let app = crate::test::build_mock_app()
            .users(vec![owner.clone(), subowner.clone()])
            .pending_projects(vec![pending_project.clone()])
            .registration_forms(vec![registration_form])
            .build();

        assert_eq!(
            Project::new(&app, pending_project, &owner)
                .await
                .unwrap()
                .unwrap_err()
                .kind(),
            NewProjectErrorKind::NotAnsweredRegistrationForm
        );
    }

    #[tokio::test]
    async fn test_new_same_owner_subowner() {
        let owner = test_model::new_general_user();
        let pending_project = test_model::new_general_pending_project(owner.id.clone());

        let app = crate::test::build_mock_app()
            .users(vec![owner.clone()])
            .pending_projects(vec![pending_project.clone()])
            .build();

        assert_eq!(
            Project::new(&app, pending_project, &owner)
                .await
                .unwrap()
                .unwrap_err()
                .kind(),
            NewProjectErrorKind::SameOwnerSubowner
        );
    }
}
