use crate::error::{UseCaseError, UseCaseResult};
use crate::model::project::{
    Project, ProjectAttribute, ProjectCategory, ProjectFromEntityInput, ProjectId,
};

use anyhow::Context;
use sos21_domain::context::project_repository::{self, ProjectRepository};
use sos21_domain::context::{ConfigContext, Login};
use sos21_domain::model::{permissions, project, user};

#[derive(Debug, Clone)]
pub struct Input {
    pub id: ProjectId,
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

    fn from_update_error(_err: project::NoUpdatePermissionError) -> Self {
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
pub async fn run<C>(ctx: &Login<C>, input: Input) -> UseCaseResult<Project, Error>
where
    C: ProjectRepository + ConfigContext + Send + Sync,
{
    let login_user = ctx.login_user();

    login_user
        .require_permissions(permissions::Permissions::UPDATE_ALL_PROJECTS)
        .map_err(|err| UseCaseError::UseCase(Error::from_permissions_error(err)))?;

    let result = ctx
        .get_project(input.id.into_entity())
        .await
        .context("Failed to get a project")?;
    let result = match result {
        Some(result) if result.project.is_visible_to(login_user) => result,
        _ => return Err(UseCaseError::UseCase(Error::NotFound)),
    };

    let project_repository::ProjectWithOwners {
        mut project,
        owner,
        subowner,
    } = result;

    if let Some(name) = input.name {
        let name = project::ProjectName::from_string(name)
            .map_err(|err| UseCaseError::UseCase(Error::from_name_error(err)))?;
        project
            .set_name(ctx, login_user, name)
            .map_err(|err| UseCaseError::UseCase(Error::from_update_error(err)))?;
    }

    if let Some(kana_name) = input.kana_name {
        let kana_name = project::ProjectKanaName::from_string(kana_name)
            .map_err(|err| UseCaseError::UseCase(Error::from_kana_name_error(err)))?;
        project
            .set_kana_name(ctx, login_user, kana_name)
            .map_err(|err| UseCaseError::UseCase(Error::from_update_error(err)))?;
    }

    if let Some(group_name) = input.group_name {
        let group_name = project::ProjectGroupName::from_string(group_name)
            .map_err(|err| UseCaseError::UseCase(Error::from_group_name_error(err)))?;
        project
            .set_group_name(ctx, login_user, group_name)
            .map_err(|err| UseCaseError::UseCase(Error::from_update_error(err)))?;
    }

    if let Some(kana_group_name) = input.kana_group_name {
        let kana_group_name = project::ProjectKanaGroupName::from_string(kana_group_name)
            .map_err(|err| UseCaseError::UseCase(Error::from_kana_group_name_error(err)))?;
        project
            .set_kana_group_name(ctx, login_user, kana_group_name)
            .map_err(|err| UseCaseError::UseCase(Error::from_update_error(err)))?;
    }

    if let Some(description) = input.description {
        let description = project::ProjectDescription::from_string(description)
            .map_err(|err| UseCaseError::UseCase(Error::from_description_error(err)))?;
        project
            .set_description(ctx, login_user, description)
            .map_err(|err| UseCaseError::UseCase(Error::from_update_error(err)))?;
    }

    if let Some(category) = input.category {
        project
            .set_category(login_user, category.into_entity())
            .map_err(|err| UseCaseError::UseCase(Error::from_update_error(err)))?;
    }

    if let Some(attributes) = input.attributes {
        let attributes = project::ProjectAttributes::from_attributes(
            attributes.into_iter().map(ProjectAttribute::into_entity),
        )
        .map_err(|err| UseCaseError::UseCase(Error::from_attributes_error(err)))?;
        project
            .set_attributes(ctx, login_user, attributes)
            .map_err(|err| UseCaseError::UseCase(Error::from_update_error(err)))?;
    }

    ctx.store_project(project.clone())
        .await
        .context("Failed to store a updated project")?;

    use_case_ensure!(
        project.is_visible_to(login_user)
            && owner.name().is_visible_to(login_user)
            && owner.kana_name().is_visible_to(login_user)
            && subowner.name().is_visible_to(login_user)
            && subowner.kana_name().is_visible_to(login_user)
    );
    Ok(Project::from_entity(ProjectFromEntityInput {
        project,
        owner_name: owner.name().clone(),
        owner_kana_name: owner.kana_name().clone(),
        subowner_name: subowner.name().clone(),
        subowner_kana_name: subowner.kana_name().clone(),
    }))
}

#[cfg(test)]
mod tests {
    use crate::model::project::ProjectId;
    use crate::{update_any_project, UseCaseError};
    use sos21_domain::test;

    // Checks that the normal user cannot update projects.
    #[tokio::test]
    async fn test_general() {
        let user = test::model::new_general_user();
        let project = test::model::new_general_project(user.id().clone());

        let app = test::build_mock_app()
            .users(vec![user.clone()])
            .projects(vec![project.clone()])
            .build()
            .login_as(user)
            .await;

        let input = update_any_project::Input {
            id: ProjectId::from_entity(project.id()),
            name: Some("新しい名前".to_string()),
            kana_name: None,
            group_name: None,
            kana_group_name: None,
            description: None,
            category: None,
            attributes: None,
        };
        assert!(matches!(
            update_any_project::run(&app, input).await,
            Err(UseCaseError::UseCase(
                update_any_project::Error::InsufficientPermissions
            ))
        ));
    }

    // Checks that the (unprivileged) committee user cannot update projects.
    #[tokio::test]
    async fn test_committee() {
        let user = test::model::new_committee_user();
        let project = test::model::new_general_project(user.id().clone());

        let app = test::build_mock_app()
            .users(vec![user.clone()])
            .projects(vec![project.clone()])
            .build()
            .login_as(user)
            .await;

        let input = update_any_project::Input {
            id: ProjectId::from_entity(project.id()),
            name: Some("新しい名前".to_string()),
            kana_name: None,
            group_name: None,
            kana_group_name: None,
            description: None,
            category: None,
            attributes: None,
        };
        assert!(matches!(
            update_any_project::run(&app, input).await,
            Err(UseCaseError::UseCase(
                update_any_project::Error::InsufficientPermissions
            ))
        ));
    }

    // Checks that the privileged committee user cannot update projects.
    #[tokio::test]
    async fn test_operator() {
        let user = test::model::new_operator_user();
        let project = test::model::new_general_project(user.id().clone());

        let app = test::build_mock_app()
            .users(vec![user.clone()])
            .projects(vec![project.clone()])
            .build()
            .login_as(user)
            .await;

        let input = update_any_project::Input {
            id: ProjectId::from_entity(project.id()),
            name: Some("新しい名前".to_string()),
            kana_name: None,
            group_name: None,
            kana_group_name: None,
            description: None,
            category: None,
            attributes: None,
        };
        assert!(matches!(
            update_any_project::run(&app, input).await,
            Err(UseCaseError::UseCase(
                update_any_project::Error::InsufficientPermissions
            ))
        ));
    }

    // Checks that the administrator can update projects.
    #[tokio::test]
    async fn test_admin() {
        let user = test::model::new_admin_user();
        let project = test::model::new_general_project(user.id().clone());

        let app = test::build_mock_app()
            .users(vec![user.clone()])
            .projects(vec![project.clone()])
            .build()
            .login_as(user)
            .await;

        let input = update_any_project::Input {
            id: ProjectId::from_entity(project.id()),
            name: Some("新しい名前".to_string()),
            kana_name: None,
            group_name: None,
            kana_group_name: None,
            description: None,
            category: None,
            attributes: None,
        };
        assert!(matches!(
            update_any_project::run(&app, input).await,
            Ok(got)
            if got.name == "新しい名前"
        ));
    }
}
