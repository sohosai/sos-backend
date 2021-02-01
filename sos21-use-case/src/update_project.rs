use crate::error::{UseCaseError, UseCaseResult};
use crate::model::project::{Project, ProjectAttribute, ProjectCategory, ProjectId};

use anyhow::Context;
use sos21_domain::context::{Login, ProjectRepository};
use sos21_domain::model::{permissions::Permissions, project};

#[derive(Debug, Clone)]
pub struct Input {
    pub id: ProjectId,
    pub display_id: Option<String>,
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
    InvalidDisplayId,
    UnavailableDisplayId,
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
    let (mut project, owner) = match result {
        Some(x) => x,
        None => return Err(UseCaseError::UseCase(Error::NotFound)),
    };

    if let Some(display_id) = input.display_id {
        let display_id = project::ProjectDisplayId::from_string(display_id)
            .map_err(|_| UseCaseError::UseCase(Error::InvalidDisplayId))?;

        project.set_display_id(ctx, display_id).await?.map_err(
            |_: project::ProjectDisplayIdNotAvailableError| {
                UseCaseError::UseCase(Error::UnavailableDisplayId)
            },
        )?;
    }

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
            && owner.name.is_visible_to(login_user)
            && owner.kana_name.is_visible_to(login_user)
    );
    Ok(Project::from_entity(
        project,
        owner.name.clone(),
        owner.kana_name.clone(),
    ))
}

#[cfg(test)]
mod tests {
    use crate::model::project::ProjectId;
    use crate::{update_project, UseCaseError};
    use sos21_domain_test as test;

    // Checks that the normal user cannot update projects.
    #[tokio::test]
    async fn test_general() {
        let user = test::model::new_general_user();
        let project = test::model::new_general_project(user.id.clone());

        let app = test::build_mock_app()
            .users(vec![user.clone()])
            .projects(vec![project.clone()])
            .build()
            .login_as(user)
            .await;

        let input = update_project::Input {
            id: ProjectId::from_entity(project.id),
            display_id: None,
            name: Some("新しい名前".to_string()),
            kana_name: None,
            group_name: None,
            kana_group_name: None,
            description: None,
            category: None,
            attributes: None,
        };
        assert!(matches!(
            update_project::run(&app, input).await,
            Err(UseCaseError::UseCase(
                update_project::Error::InsufficientPermissions
            ))
        ));
    }

    // Checks that the (unprivileged) committee user cannot update projects.
    #[tokio::test]
    async fn test_committee() {
        let user = test::model::new_committee_user();
        let project = test::model::new_general_project(user.id.clone());

        let app = test::build_mock_app()
            .users(vec![user.clone()])
            .projects(vec![project.clone()])
            .build()
            .login_as(user)
            .await;

        let input = update_project::Input {
            id: ProjectId::from_entity(project.id),
            display_id: None,
            name: Some("新しい名前".to_string()),
            kana_name: None,
            group_name: None,
            kana_group_name: None,
            description: None,
            category: None,
            attributes: None,
        };
        assert!(matches!(
            update_project::run(&app, input).await,
            Err(UseCaseError::UseCase(
                update_project::Error::InsufficientPermissions
            ))
        ));
    }

    // Checks that the privileged committee user cannot update projects.
    #[tokio::test]
    async fn test_operator() {
        let user = test::model::new_operator_user();
        let project = test::model::new_general_project(user.id.clone());

        let app = test::build_mock_app()
            .users(vec![user.clone()])
            .projects(vec![project.clone()])
            .build()
            .login_as(user)
            .await;

        let input = update_project::Input {
            id: ProjectId::from_entity(project.id),
            display_id: None,
            name: Some("新しい名前".to_string()),
            kana_name: None,
            group_name: None,
            kana_group_name: None,
            description: None,
            category: None,
            attributes: None,
        };
        assert!(matches!(
            update_project::run(&app, input).await,
            Err(UseCaseError::UseCase(
                update_project::Error::InsufficientPermissions
            ))
        ));
    }

    // Checks that the administrator can update projects.
    #[tokio::test]
    async fn test_admin() {
        let user = test::model::new_admin_user();
        let project = test::model::new_general_project(user.id.clone());

        let app = test::build_mock_app()
            .users(vec![user.clone()])
            .projects(vec![project.clone()])
            .build()
            .login_as(user)
            .await;

        let input = update_project::Input {
            id: ProjectId::from_entity(project.id),
            display_id: None,
            name: Some("新しい名前".to_string()),
            kana_name: None,
            group_name: None,
            kana_group_name: None,
            description: None,
            category: None,
            attributes: None,
        };
        assert!(matches!(
            update_project::run(&app, input).await,
            Ok(got)
            if got.name == "新しい名前"
        ));
    }

    // Checks that the administrator cannot update project with the duplicated display ID.
    #[tokio::test]
    async fn test_duplicated_display_id() {
        let user = test::model::new_admin_user();
        let project1 = test::model::new_general_project(user.id.clone());
        let project2 = test::model::new_general_project(user.id.clone());

        let app = test::build_mock_app()
            .users(vec![user.clone()])
            .projects(vec![project1.clone(), project2.clone()])
            .build()
            .login_as(user)
            .await;

        let input = update_project::Input {
            id: ProjectId::from_entity(project1.id),
            display_id: Some(project2.display_id.into_string()),
            name: None,
            kana_name: None,
            group_name: None,
            kana_group_name: None,
            description: None,
            category: None,
            attributes: None,
        };
        assert!(matches!(
            update_project::run(&app, input).await,
            Err(UseCaseError::UseCase(
                update_project::Error::UnavailableDisplayId
            ))
        ));
    }
}
