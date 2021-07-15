use crate::error::{UseCaseError, UseCaseResult};
use crate::model::pending_project::{PendingProject, PendingProjectId};
use crate::model::project::{ProjectAttribute, ProjectCategory};

use anyhow::Context;
use sos21_domain::context::pending_project_repository::PendingProjectRepository;
use sos21_domain::context::{ConfigContext, Login};
use sos21_domain::model::{pending_project, permissions, project, user};

#[derive(Debug, Clone)]
pub struct Input {
    pub id: PendingProjectId,
    pub name: Option<String>,
    pub kana_name: Option<String>,
    pub group_name: Option<String>,
    pub kana_group_name: Option<String>,
    pub description: Option<String>,
    pub category: Option<ProjectCategory>,
    pub attributes: Option<Vec<ProjectAttribute>>,
}

#[derive(Debug, Clone)]
pub enum Error {
    NotFound,
    InvalidName,
    InvalidKanaName,
    InvalidGroupName,
    InvalidKanaGroupName,
    InvalidDescription,
    DuplicatedAttributes,
    InsufficientPermissions,
}

impl Error {
    fn from_permissions_error(_err: user::RequirePermissionsError) -> Self {
        Error::InsufficientPermissions
    }

    fn from_update_error(_err: pending_project::NoUpdatePermissionError) -> Self {
        Error::InsufficientPermissions
    }

    fn from_name_error(_err: project::name::NameError) -> Self {
        Error::InvalidName
    }

    fn from_kana_name_error(_err: project::name::KanaNameError) -> Self {
        Error::InvalidKanaName
    }

    fn from_group_name_error(_err: project::name::GroupNameError) -> Self {
        Error::InvalidGroupName
    }

    fn from_kana_group_name_error(_err: project::name::KanaGroupNameError) -> Self {
        Error::InvalidKanaGroupName
    }

    fn from_description_error(_err: project::description::DescriptionError) -> Self {
        Error::InvalidDescription
    }

    fn from_attributes_error(_err: project::attribute::DuplicatedAttributesError) -> Self {
        Error::DuplicatedAttributes
    }
}

#[tracing::instrument(skip(ctx))]
pub async fn run<C>(ctx: &Login<C>, input: Input) -> UseCaseResult<PendingProject, Error>
where
    C: PendingProjectRepository + ConfigContext + Send + Sync,
{
    let login_user = ctx.login_user();

    login_user
        .require_permissions(permissions::Permissions::UPDATE_ALL_PENDING_PROJECTS)
        .map_err(|err| UseCaseError::UseCase(Error::from_permissions_error(err)))?;

    let result = ctx
        .get_pending_project(input.id.into_entity())
        .await
        .context("Failed to get a pending project")?;
    let mut pending_project = match result {
        Some(result) if result.pending_project.is_visible_to(login_user) => result.pending_project,
        _ => return Err(UseCaseError::UseCase(Error::NotFound)),
    };

    if let Some(name) = input.name {
        let name = project::ProjectName::from_string(name)
            .map_err(|err| UseCaseError::UseCase(Error::from_name_error(err)))?;
        pending_project
            .set_name(ctx, login_user, name)
            .map_err(|err| UseCaseError::UseCase(Error::from_update_error(err)))?;
    }

    if let Some(kana_name) = input.kana_name {
        let kana_name = project::ProjectKanaName::from_string(kana_name)
            .map_err(|err| UseCaseError::UseCase(Error::from_kana_name_error(err)))?;
        pending_project
            .set_kana_name(ctx, login_user, kana_name)
            .map_err(|err| UseCaseError::UseCase(Error::from_update_error(err)))?;
    }

    if let Some(group_name) = input.group_name {
        let group_name = project::ProjectGroupName::from_string(group_name)
            .map_err(|err| UseCaseError::UseCase(Error::from_group_name_error(err)))?;
        pending_project
            .set_group_name(ctx, login_user, group_name)
            .map_err(|err| UseCaseError::UseCase(Error::from_update_error(err)))?;
    }

    if let Some(kana_group_name) = input.kana_group_name {
        let kana_group_name = project::ProjectKanaGroupName::from_string(kana_group_name)
            .map_err(|err| UseCaseError::UseCase(Error::from_kana_group_name_error(err)))?;
        pending_project
            .set_kana_group_name(ctx, login_user, kana_group_name)
            .map_err(|err| UseCaseError::UseCase(Error::from_update_error(err)))?;
    }

    if let Some(description) = input.description {
        let description = project::ProjectDescription::from_string(description)
            .map_err(|err| UseCaseError::UseCase(Error::from_description_error(err)))?;
        pending_project
            .set_description(ctx, login_user, description)
            .map_err(|err| UseCaseError::UseCase(Error::from_update_error(err)))?;
    }

    if let Some(attributes) = input.attributes {
        let attributes = project::ProjectAttributes::from_attributes(
            attributes.into_iter().map(ProjectAttribute::into_entity),
        )
        .map_err(|err| UseCaseError::UseCase(Error::from_attributes_error(err)))?;
        pending_project
            .set_attributes(ctx, login_user, attributes)
            .map_err(|err| UseCaseError::UseCase(Error::from_update_error(err)))?;
    }

    ctx.store_pending_project(pending_project.clone())
        .await
        .context("Failed to store a updated pending project")?;

    use_case_ensure!(pending_project.is_visible_to(login_user));
    Ok(PendingProject::from_entity(pending_project))
}

#[cfg(test)]
mod tests {
    use crate::model::pending_project::PendingProjectId;
    use crate::{update_any_pending_project, UseCaseError};
    use sos21_domain::test;

    // Checks that the normal user cannot update pending projects.
    #[tokio::test]
    async fn test_general() {
        let user = test::model::new_general_user();
        let pending_project = test::model::new_general_pending_project(user.id().clone());

        let app = test::build_mock_app()
            .users(vec![user.clone()])
            .pending_projects(vec![pending_project.clone()])
            .build()
            .login_as(user)
            .await;

        let input = update_any_pending_project::Input {
            id: PendingProjectId::from_entity(pending_project.id()),
            name: Some("新しい名前".to_string()),
            kana_name: None,
            group_name: None,
            kana_group_name: None,
            description: None,
            category: None,
            attributes: None,
        };
        assert!(matches!(
            update_any_pending_project::run(&app, input).await,
            Err(UseCaseError::UseCase(
                update_any_pending_project::Error::InsufficientPermissions
            ))
        ));
    }

    // Checks that the (unprivileged) committee user cannot update pending projects.
    #[tokio::test]
    async fn test_committee() {
        let user = test::model::new_committee_user();
        let pending_project = test::model::new_general_pending_project(user.id().clone());

        let app = test::build_mock_app()
            .users(vec![user.clone()])
            .pending_projects(vec![pending_project.clone()])
            .build()
            .login_as(user)
            .await;

        let input = update_any_pending_project::Input {
            id: PendingProjectId::from_entity(pending_project.id()),
            name: Some("新しい名前".to_string()),
            kana_name: None,
            group_name: None,
            kana_group_name: None,
            description: None,
            category: None,
            attributes: None,
        };
        assert!(matches!(
            update_any_pending_project::run(&app, input).await,
            Err(UseCaseError::UseCase(
                update_any_pending_project::Error::InsufficientPermissions
            ))
        ));
    }

    // Checks that the privileged committee user cannot update pending projects.
    #[tokio::test]
    async fn test_operator() {
        let user = test::model::new_operator_user();
        let pending_project = test::model::new_general_pending_project(user.id().clone());

        let app = test::build_mock_app()
            .users(vec![user.clone()])
            .pending_projects(vec![pending_project.clone()])
            .build()
            .login_as(user)
            .await;

        let input = update_any_pending_project::Input {
            id: PendingProjectId::from_entity(pending_project.id()),
            name: Some("新しい名前".to_string()),
            kana_name: None,
            group_name: None,
            kana_group_name: None,
            description: None,
            category: None,
            attributes: None,
        };
        assert!(matches!(
            update_any_pending_project::run(&app, input).await,
            Err(UseCaseError::UseCase(
                update_any_pending_project::Error::InsufficientPermissions
            ))
        ));
    }

    // Checks that the administrator can update projects.
    #[tokio::test]
    async fn test_admin() {
        let user = test::model::new_admin_user();
        let pending_project = test::model::new_general_pending_project(user.id().clone());

        let app = test::build_mock_app()
            .users(vec![user.clone()])
            .pending_projects(vec![pending_project.clone()])
            .build()
            .login_as(user)
            .await;

        let input = update_any_pending_project::Input {
            id: PendingProjectId::from_entity(pending_project.id()),
            name: Some("新しい名前".to_string()),
            kana_name: None,
            group_name: None,
            kana_group_name: None,
            description: None,
            category: None,
            attributes: None,
        };
        assert!(matches!(
            update_any_pending_project::run(&app, input).await,
            Ok(got)
            if got.name == "新しい名前"
        ));
    }
}
