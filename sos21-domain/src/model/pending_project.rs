use crate::context::ConfigContext;
use crate::model::date_time::DateTime;
use crate::model::project::{
    ProjectAttributes, ProjectCategory, ProjectDescription, ProjectGroupName, ProjectKanaGroupName,
    ProjectKanaName, ProjectName,
};
use crate::model::user::{User, UserAssignment, UserId};

use thiserror::Error;
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
pub struct PendingProjectContent {
    pub id: PendingProjectId,
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
pub struct PendingProject {
    content: PendingProjectContent,
    // TODO: Query every time to make sure this is up to date
    owner_id: UserId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NewPendingProjectErrorKind {
    AlreadyProjectOwnerOwner,
    AlreadyProjectSubownerOwner,
    AlreadyPendingProjectOwnerOwner,
    OutOfCreationPeriod,
}

#[derive(Debug, Clone, Error)]
#[error("failed to create a pending project")]
pub struct NewPendingProjectError {
    kind: NewPendingProjectErrorKind,
}

impl NewPendingProjectError {
    pub fn kind(&self) -> NewPendingProjectErrorKind {
        self.kind
    }
}

impl PendingProject {
    #[allow(clippy::too_many_arguments)]
    pub fn new<C>(
        ctx: C,
        owner: &User,
        name: ProjectName,
        kana_name: ProjectKanaName,
        group_name: ProjectGroupName,
        kana_group_name: ProjectKanaGroupName,
        description: ProjectDescription,
        category: ProjectCategory,
        attributes: ProjectAttributes,
    ) -> Result<Self, NewPendingProjectError>
    where
        C: ConfigContext,
    {
        let created_at = DateTime::now();
        if !ctx.project_creation_period().contains(created_at) {
            return Err(NewPendingProjectError {
                kind: NewPendingProjectErrorKind::OutOfCreationPeriod,
            });
        }

        if let Some(assignment) = owner.assignment() {
            let kind = match assignment {
                UserAssignment::ProjectOwner(_) => {
                    NewPendingProjectErrorKind::AlreadyProjectOwnerOwner
                }
                UserAssignment::ProjectSubowner(_) => {
                    NewPendingProjectErrorKind::AlreadyProjectSubownerOwner
                }
                UserAssignment::PendingProjectOwner(_) => {
                    NewPendingProjectErrorKind::AlreadyPendingProjectOwnerOwner
                }
            };
            return Err(NewPendingProjectError { kind });
        }

        Ok(PendingProject::from_content(
            PendingProjectContent {
                id: PendingProjectId::from_uuid(Uuid::new_v4()),
                created_at,
                name,
                kana_name,
                group_name,
                kana_group_name,
                description,
                category,
                attributes,
            },
            owner.id().clone(),
        ))
    }

    pub fn id(&self) -> PendingProjectId {
        self.content.id
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

    /// Restore `PendingProject` from `PendingProjectContent`.
    ///
    /// This is intended to be used when the data is taken out of the implementation by [`PendingProject::into_content`]
    /// for persistence, internal serialization, etc.
    /// Use [`PendingProject::new`] to create a project.
    pub fn from_content(content: PendingProjectContent, owner_id: UserId) -> Self {
        PendingProject { content, owner_id }
    }

    /// Convert `PendingProject` into `PendingProjectContent`.
    pub fn into_content(self) -> PendingProjectContent {
        self.content
    }

    pub fn is_visible_to(&self, _user: &User) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::{NewPendingProjectErrorKind, PendingProject};

    use crate::model::project::{ProjectAttributes, ProjectCategory};
    use crate::test::model as test_model;

    #[tokio::test]
    async fn test_new_already_project_owner() {
        let mut owner = test_model::new_general_user();
        let project = test_model::new_general_project(owner.id().clone());
        owner.assign_project_owner(&project).unwrap();

        assert_eq!(
            PendingProject::new(
                crate::test::build_mock_app().build(),
                &owner,
                test_model::mock_project_name(),
                test_model::mock_project_kana_name(),
                test_model::mock_project_group_name(),
                test_model::mock_project_kana_group_name(),
                test_model::mock_project_description(),
                ProjectCategory::General,
                ProjectAttributes::from_attributes(vec![]).unwrap()
            )
            .unwrap_err()
            .kind(),
            NewPendingProjectErrorKind::AlreadyProjectOwnerOwner
        );
    }

    #[tokio::test]
    async fn test_new_already_project_subowner() {
        let mut owner = test_model::new_general_user();
        let user = test_model::new_general_user();
        let project =
            test_model::new_general_project_with_subowner(user.id().clone(), owner.id().clone());
        owner.assign_project_subowner(&project).unwrap();

        assert_eq!(
            PendingProject::new(
                crate::test::build_mock_app().build(),
                &owner,
                test_model::mock_project_name(),
                test_model::mock_project_kana_name(),
                test_model::mock_project_group_name(),
                test_model::mock_project_kana_group_name(),
                test_model::mock_project_description(),
                ProjectCategory::General,
                ProjectAttributes::from_attributes(vec![]).unwrap()
            )
            .unwrap_err()
            .kind(),
            NewPendingProjectErrorKind::AlreadyProjectSubownerOwner
        );
    }

    #[tokio::test]
    async fn test_new_already_pending_project_owner() {
        let mut owner = test_model::new_general_user();
        let pending_project = test_model::new_general_pending_project(owner.id().clone());
        owner
            .assign_pending_project_owner(&pending_project)
            .unwrap();

        assert_eq!(
            PendingProject::new(
                crate::test::build_mock_app().build(),
                &owner,
                test_model::mock_project_name(),
                test_model::mock_project_kana_name(),
                test_model::mock_project_group_name(),
                test_model::mock_project_kana_group_name(),
                test_model::mock_project_description(),
                ProjectCategory::General,
                ProjectAttributes::from_attributes(vec![]).unwrap()
            )
            .unwrap_err()
            .kind(),
            NewPendingProjectErrorKind::AlreadyPendingProjectOwnerOwner
        );
    }

    // TODO: test new out of period
}
