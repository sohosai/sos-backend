use crate::error::{UseCaseError, UseCaseResult};
use crate::model::project::{
    Project, ProjectAttribute, ProjectCategory, ProjectFromEntityInput, ProjectId,
};

use anyhow::Context;
use sos21_domain::context::project_repository::{self, ProjectRepository};
use sos21_domain::context::Login;
use sos21_domain::model::{permissions::Permissions, project};

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

#[tracing::instrument(skip(ctx))]
pub async fn run<C>(ctx: &Login<C>, input: Input) -> UseCaseResult<Project, Error>
where
    C: ProjectRepository + Send + Sync,
{
    let login_user = ctx.login_user();

    if login_user
        .require_permissions(Permissions::UPDATE_ALL_PROJECTS)
        .is_err()
    {
        return Err(UseCaseError::UseCase(Error::InsufficientPermissions));
    }

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
            .map_err(|_| UseCaseError::UseCase(Error::InvalidName))?;
        project.set_name(name);
    }

    if let Some(kana_name) = input.kana_name {
        let kana_name = project::ProjectKanaName::from_string(kana_name)
            .map_err(|_| UseCaseError::UseCase(Error::InvalidKanaName))?;
        project.set_kana_name(kana_name);
    }

    if let Some(group_name) = input.group_name {
        let group_name = project::ProjectGroupName::from_string(group_name)
            .map_err(|_| UseCaseError::UseCase(Error::InvalidGroupName))?;
        project.set_group_name(group_name);
    }

    if let Some(kana_group_name) = input.kana_group_name {
        let kana_group_name = project::ProjectKanaGroupName::from_string(kana_group_name)
            .map_err(|_| UseCaseError::UseCase(Error::InvalidKanaGroupName))?;
        project.set_kana_group_name(kana_group_name);
    }

    if let Some(description) = input.description {
        let description = project::ProjectDescription::from_string(description)
            .map_err(|_| UseCaseError::UseCase(Error::InvalidDescription))?;
        project.set_description(description);
    }

    if let Some(attributes) = input.attributes {
        let attributes = project::ProjectAttributes::from_attributes(
            attributes.into_iter().map(ProjectAttribute::into_entity),
        )
        .map_err(|_: project::attribute::DuplicatedAttributesError| {
            UseCaseError::UseCase(Error::DuplicatedAttributes)
        })?;
        project.set_attributes(attributes);
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
