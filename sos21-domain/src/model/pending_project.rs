use crate::context::ConfigContext;
use crate::model::date_time::DateTime;
use crate::model::permissions::Permissions;
use crate::model::project::{
    ProjectAttribute, ProjectAttributes, ProjectCategory, ProjectDescription, ProjectGroupName,
    ProjectKanaGroupName, ProjectKanaName, ProjectName,
};
use crate::model::user::{self, User, UserAssignment, UserId};

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
    pub updated_at: DateTime,
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
    ArtisticStageProject,
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
        if !ctx
            .project_creation_period_for(category)
            .contains(created_at)
        {
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

        if (category == ProjectCategory::StageOnline || category == ProjectCategory::StagePhysical)
            && attributes.contains(ProjectAttribute::Artistic)
        {
            return Err(NewPendingProjectError {
                kind: NewPendingProjectErrorKind::ArtisticStageProject,
            });
        }

        Ok(PendingProject::from_content(
            PendingProjectContent {
                id: PendingProjectId::from_uuid(Uuid::new_v4()),
                created_at,
                updated_at: created_at,
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

    pub fn updated_at(&self) -> DateTime {
        self.content.updated_at
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

#[derive(Debug, Clone, Error)]
#[error("insufficient permissions to update pending project")]
pub struct NoUpdatePermissionError {
    _priv: (),
}

impl NoUpdatePermissionError {
    fn from_permissions_error(_err: user::RequirePermissionsError) -> Self {
        NoUpdatePermissionError { _priv: () }
    }
}

impl PendingProject {
    fn require_update_permission<C>(
        &self,
        ctx: C,
        now: DateTime,
        user: &User,
    ) -> Result<(), NoUpdatePermissionError>
    where
        C: ConfigContext,
    {
        let permission = if &self.owner_id == user.id()
            && ctx
                .project_creation_period_for(self.category())
                .contains(now)
        {
            Permissions::UPDATE_OWNING_PENDING_PROJECTS_IN_PERIOD
        } else {
            Permissions::UPDATE_ALL_PENDING_PROJECTS
        };

        user.require_permissions(permission)
            .map_err(NoUpdatePermissionError::from_permissions_error)
    }

    pub fn set_name<C>(
        &mut self,
        ctx: C,
        user: &User,
        name: ProjectName,
    ) -> Result<(), NoUpdatePermissionError>
    where
        C: ConfigContext,
    {
        let now = DateTime::now();
        self.require_update_permission(ctx, now, user)?;
        self.content.name = name;
        self.content.updated_at = now;
        Ok(())
    }

    pub fn set_kana_name<C>(
        &mut self,
        ctx: C,
        user: &User,
        kana_name: ProjectKanaName,
    ) -> Result<(), NoUpdatePermissionError>
    where
        C: ConfigContext,
    {
        let now = DateTime::now();
        self.require_update_permission(ctx, now, user)?;
        self.content.kana_name = kana_name;
        self.content.updated_at = now;
        Ok(())
    }

    pub fn set_group_name<C>(
        &mut self,
        ctx: C,
        user: &User,
        group_name: ProjectGroupName,
    ) -> Result<(), NoUpdatePermissionError>
    where
        C: ConfigContext,
    {
        let now = DateTime::now();
        self.require_update_permission(ctx, now, user)?;
        self.content.group_name = group_name;
        self.content.updated_at = now;
        Ok(())
    }

    pub fn set_kana_group_name<C>(
        &mut self,
        ctx: C,
        user: &User,
        kana_group_name: ProjectKanaGroupName,
    ) -> Result<(), NoUpdatePermissionError>
    where
        C: ConfigContext,
    {
        let now = DateTime::now();
        self.require_update_permission(ctx, now, user)?;
        self.content.kana_group_name = kana_group_name;
        self.content.updated_at = now;
        Ok(())
    }

    pub fn set_description<C>(
        &mut self,
        ctx: C,
        user: &User,
        description: ProjectDescription,
    ) -> Result<(), NoUpdatePermissionError>
    where
        C: ConfigContext,
    {
        let now = DateTime::now();
        self.require_update_permission(ctx, now, user)?;
        self.content.description = description;
        self.content.updated_at = now;
        Ok(())
    }

    pub fn set_category(
        &mut self,
        user: &User,
        category: ProjectCategory,
    ) -> Result<(), NoUpdatePermissionError> {
        user.require_permissions(Permissions::UPDATE_PENDING_PROJECT_CATEGORY)
            .map_err(NoUpdatePermissionError::from_permissions_error)?;
        self.content.category = category;
        self.content.updated_at = DateTime::now();
        Ok(())
    }

    pub fn set_attributes<C>(
        &mut self,
        ctx: C,
        user: &User,
        attributes: ProjectAttributes,
    ) -> Result<(), NoUpdatePermissionError>
    where
        C: ConfigContext,
    {
        let now = DateTime::now();
        self.require_update_permission(ctx, now, user)?;
        self.content.attributes = attributes;
        self.content.updated_at = now;
        Ok(())
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
        let project = test_model::new_general_online_project(owner.id().clone());
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
                ProjectCategory::GeneralOnline,
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
        let project = test_model::new_general_online_project_with_subowner(
            user.id().clone(),
            owner.id().clone(),
        );
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
                ProjectCategory::GeneralOnline,
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
        let pending_project = test_model::new_general_online_pending_project(owner.id().clone());
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
                ProjectCategory::GeneralOnline,
                ProjectAttributes::from_attributes(vec![]).unwrap()
            )
            .unwrap_err()
            .kind(),
            NewPendingProjectErrorKind::AlreadyPendingProjectOwnerOwner
        );
    }

    // TODO: test new out of period
    // TODO: test set_* permissions and period
}
