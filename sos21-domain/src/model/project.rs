use std::convert::TryInto;

use crate::context::{
    ProjectRepository, RegistrationFormAnswerRepository, RegistrationFormRepository,
};
use crate::model::date_time::DateTime;
use crate::model::pending_project::PendingProject;
use crate::model::permissions::Permissions;
use crate::model::user::{User, UserAssignment, UserId};

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
    pub name: ProjectName,
    pub kana_name: ProjectKanaName,
    pub group_name: ProjectGroupName,
    pub kana_group_name: ProjectKanaGroupName,
    pub description: ProjectDescription,
    pub category: ProjectCategory,
    pub attributes: ProjectAttributes,
}

#[derive(Debug, Clone)]
pub struct Project {
    content: ProjectContent,
    owner_id: UserId,
    subowner_id: UserId,
}

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
    AlreadyProjectOwnerSubowner,
    AlreadyProjectSubownerSubowner,
    AlreadyPendingProjectOwnerSubowner,
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
            .count_registration_forms_by_pending_project(pending_project.id())
            .await
            .context("Failed to count registration forms")?;
        let answers_count = ctx
            .count_registration_form_answers_by_pending_project(pending_project.id())
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

        if let Some(assignment) = subowner.assignment() {
            let kind = match assignment {
                UserAssignment::ProjectOwner(_) => NewProjectErrorKind::AlreadyProjectOwnerSubowner,
                UserAssignment::ProjectSubowner(_) => {
                    NewProjectErrorKind::AlreadyProjectSubownerSubowner
                }
                UserAssignment::PendingProjectOwner(_) => {
                    NewProjectErrorKind::AlreadyPendingProjectOwnerSubowner
                }
            };
            return Ok(Err(NewProjectError { kind }));
        }

        Ok(Project::from_content(
            ProjectContent {
                id: ProjectId::from_uuid(Uuid::new_v4()),
                created_at: DateTime::now(),
                index,
                name: pending_project.name().clone(),
                kana_name: pending_project.kana_name().clone(),
                group_name: pending_project.group_name().clone(),
                kana_group_name: pending_project.kana_group_name().clone(),
                description: pending_project.description().clone(),
                category: pending_project.category(),
                attributes: pending_project.attributes().clone(),
            },
            pending_project.owner_id().clone(),
            subowner.id().clone(),
        )
        .map_err(NewProjectError::from_content_error))
    }

    /// Restore `Project` from `ProjectContent`.
    ///
    /// This is intended to be used when the data is taken out of the implementation by [`Project::into_content`]
    /// for persistence, internal serialization, etc.
    /// Use [`Project::new`] to create a project.
    pub fn from_content(
        content: ProjectContent,
        owner_id: UserId,
        subowner_id: UserId,
    ) -> Result<Self, SameOwnerSubownerError> {
        if owner_id == subowner_id {
            return Err(SameOwnerSubownerError { _priv: () });
        }

        Ok(Project {
            content,
            owner_id,
            subowner_id,
        })
    }

    /// Convert `Project` into `ProjectContent`.
    pub fn into_content(self) -> ProjectContent {
        self.content
    }

    pub fn id(&self) -> ProjectId {
        self.content.id
    }

    pub fn index(&self) -> ProjectIndex {
        self.content.index
    }

    pub fn created_at(&self) -> DateTime {
        self.content.created_at
    }

    pub fn name(&self) -> &ProjectName {
        &self.content.name
    }

    pub fn kana_name(&self) -> &ProjectKanaName {
        &self.content.kana_name
    }

    pub fn group_name(&self) -> &ProjectGroupName {
        &self.content.group_name
    }

    pub fn kana_group_name(&self) -> &ProjectKanaGroupName {
        &self.content.kana_group_name
    }

    pub fn description(&self) -> &ProjectDescription {
        &self.content.description
    }

    pub fn category(&self) -> ProjectCategory {
        self.content.category
    }

    pub fn attributes(&self) -> &ProjectAttributes {
        &self.content.attributes
    }

    pub fn owner_id(&self) -> &UserId {
        &self.owner_id
    }

    pub fn subowner_id(&self) -> &UserId {
        &self.subowner_id
    }

    pub fn is_visible_to(&self, user: &User) -> bool {
        if &self.owner_id == user.id() || &self.subowner_id == user.id() {
            return true;
        }

        user.permissions().contains(Permissions::READ_ALL_PROJECTS)
    }

    pub fn kind(&self) -> ProjectKind {
        ProjectKind {
            is_cooking: self.content.category == ProjectCategory::Cooking,
            is_outdoor: self.content.attributes.contains(ProjectAttribute::Outdoor),
        }
    }

    pub fn code(&self) -> ProjectCode {
        ProjectCode {
            kind: self.kind(),
            index: self.content.index,
        }
    }

    pub fn set_name(&mut self, name: ProjectName) {
        self.content.name = name;
    }

    pub fn set_kana_name(&mut self, kana_name: ProjectKanaName) {
        self.content.kana_name = kana_name;
    }

    pub fn set_group_name(&mut self, group_name: ProjectGroupName) {
        self.content.group_name = group_name;
    }

    pub fn set_kana_group_name(&mut self, kana_group_name: ProjectKanaGroupName) {
        self.content.kana_group_name = kana_group_name;
    }

    pub fn set_description(&mut self, description: ProjectDescription) {
        self.content.description = description;
    }

    pub fn set_category(&mut self, category: ProjectCategory) {
        self.content.category = category;
    }

    pub fn set_attributes(&mut self, attributes: ProjectAttributes) {
        self.content.attributes = attributes;
    }
}

#[cfg(test)]
mod tests {
    use super::{NewProjectErrorKind, Project};
    use crate::test::model as test_model;

    #[test]
    fn test_visibility_general_owner() {
        let user = test_model::new_general_user();
        let project = test_model::new_general_project(user.id().clone());
        assert!(project.is_visible_to(&user));
    }

    #[test]
    fn test_visibility_general_subowner() {
        let user = test_model::new_general_user();
        let project = test_model::new_general_project_with_subowner(
            test_model::new_user_id(),
            user.id().clone(),
        );
        assert!(project.is_visible_to(&user));
    }

    #[test]
    fn test_visibility_general_other() {
        let user = test_model::new_general_user();
        let other = test_model::new_general_user();
        let project = test_model::new_general_project(other.id().clone());
        assert!(!project.is_visible_to(&user));
    }

    #[test]
    fn test_visibility_committee_other() {
        let user = test_model::new_committee_user();
        let other = test_model::new_general_user();
        let project = test_model::new_general_project(other.id().clone());
        assert!(project.is_visible_to(&user));
    }

    #[test]
    fn test_visibility_operator_other() {
        let user = test_model::new_operator_user();
        let other = test_model::new_general_user();
        let project = test_model::new_general_project(other.id().clone());
        assert!(project.is_visible_to(&user));
    }

    #[tokio::test]
    async fn test_new_ok() {
        let owner = test_model::new_general_user();
        let subowner = test_model::new_general_user();
        let pending_project = test_model::new_general_pending_project(owner.id().clone());

        let app = crate::test::build_mock_app()
            .users(vec![owner.clone(), subowner.clone()])
            .pending_projects(vec![pending_project.clone()])
            .build();

        let project = Project::new(&app, pending_project, &subowner)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(project.owner_id(), owner.id());
        assert_eq!(project.subowner_id(), subowner.id());
    }

    #[tokio::test]
    async fn test_new_not_answered() {
        let owner = test_model::new_general_user();
        let subowner = test_model::new_general_user();
        let pending_project = test_model::new_general_pending_project(owner.id().clone());

        let operator = test_model::new_general_user();
        let registration_form = test_model::new_registration_form(operator.id().clone());

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
        let pending_project = test_model::new_general_pending_project(owner.id().clone());

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

    #[tokio::test]
    async fn test_new_already_project_owner() {
        let owner = test_model::new_general_user();
        let mut subowner = test_model::new_general_user();
        let project = test_model::new_general_project(subowner.id().clone());
        subowner.assign_project_owner(&project).unwrap();

        let pending_project = test_model::new_general_pending_project(owner.id().clone());

        let app = crate::test::build_mock_app()
            .users(vec![owner.clone(), subowner.clone()])
            .projects(vec![project.clone()])
            .pending_projects(vec![pending_project.clone()])
            .build();

        assert_eq!(
            Project::new(&app, pending_project, &subowner)
                .await
                .unwrap()
                .unwrap_err()
                .kind(),
            NewProjectErrorKind::AlreadyProjectOwnerSubowner
        );
    }

    #[tokio::test]
    async fn test_new_already_project_subowner() {
        let owner = test_model::new_general_user();
        let user = test_model::new_general_user();
        let mut subowner = test_model::new_general_user();
        let project =
            test_model::new_general_project_with_subowner(user.id().clone(), subowner.id().clone());
        subowner.assign_project_subowner(&project).unwrap();

        let pending_project = test_model::new_general_pending_project(owner.id().clone());

        let app = crate::test::build_mock_app()
            .users(vec![user, owner.clone(), subowner.clone()])
            .projects(vec![project.clone()])
            .pending_projects(vec![pending_project.clone()])
            .build();

        assert_eq!(
            Project::new(&app, pending_project, &subowner)
                .await
                .unwrap()
                .unwrap_err()
                .kind(),
            NewProjectErrorKind::AlreadyProjectSubownerSubowner
        );
    }

    #[tokio::test]
    async fn test_new_already_pending_project_owner() {
        let owner = test_model::new_general_user();
        let mut subowner = test_model::new_general_user();
        let pending_project1 = test_model::new_general_pending_project(subowner.id().clone());
        subowner
            .assign_pending_project_owner(&pending_project1)
            .unwrap();

        let pending_project2 = test_model::new_general_pending_project(owner.id().clone());

        let app = crate::test::build_mock_app()
            .users(vec![owner.clone(), subowner.clone()])
            .pending_projects(vec![pending_project1.clone(), pending_project2.clone()])
            .build();

        assert_eq!(
            Project::new(&app, pending_project2, &subowner)
                .await
                .unwrap()
                .unwrap_err()
                .kind(),
            NewProjectErrorKind::AlreadyPendingProjectOwnerSubowner
        );
    }
}
