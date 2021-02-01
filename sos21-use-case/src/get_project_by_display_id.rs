use crate::error::{UseCaseError, UseCaseResult};
use crate::model::project::Project;

use anyhow::Context;
use sos21_domain::context::{Login, ProjectRepository};
use sos21_domain::model::project;

#[derive(Debug, Clone)]
pub enum Error {
    InvalidDisplayId,
    NotFound,
}

#[tracing::instrument(skip(ctx))]
pub async fn run<C>(ctx: &Login<C>, display_id: String) -> UseCaseResult<Project, Error>
where
    Login<C>: ProjectRepository,
{
    let display_id = project::ProjectDisplayId::from_string(display_id)
        .map_err(|_| UseCaseError::UseCase(Error::InvalidDisplayId))?;

    let result = ctx
        .find_project_by_display_id(display_id)
        .await
        .context("Failed to get a project")?;
    let (project, owner) = match result {
        Some(x) => x,
        None => return Err(UseCaseError::UseCase(Error::NotFound)),
    };

    let login_user = ctx.login_user();
    if !project.is_visible_to(login_user)
        || !owner.name.is_visible_to(login_user)
        || !owner.kana_name.is_visible_to(login_user)
    {
        return Err(UseCaseError::UseCase(Error::NotFound));
    }

    Ok(Project::from_entity(project, owner.name, owner.kana_name))
}

#[cfg(test)]
mod tests {
    use crate::model::project::ProjectId;
    use crate::{get_project_by_display_id, UseCaseError};
    use sos21_domain_test as test;

    // Checks that the normal user cannot read the others' project.
    #[tokio::test]
    async fn test_general_other() {
        let user = test::model::new_general_user();
        let other = test::model::new_general_user();
        let project_other = test::model::new_general_project(other.id.clone());

        let app = test::build_mock_app()
            .users(vec![user.clone(), other.clone()])
            .projects(vec![project_other.clone()])
            .build()
            .login_as(user.clone())
            .await;

        assert!(matches!(
            get_project_by_display_id::run(&app, project_other.display_id.into_string()).await,
            Err(UseCaseError::UseCase(
                get_project_by_display_id::Error::NotFound
            ))
        ));
    }

    // Checks that the normal user can read the owning project.
    #[tokio::test]
    async fn test_general_owner() {
        let user = test::model::new_general_user();
        let project = test::model::new_general_project(user.id.clone());

        let app = test::build_mock_app()
            .users(vec![user.clone()])
            .projects(vec![project.clone()])
            .build()
            .login_as(user.clone())
            .await;

        assert!(matches!(
            get_project_by_display_id::run(&app, project.display_id.into_string()).await,
            Ok(got)
            if got.id == ProjectId::from_entity(project.id) && got.name == project.name.into_string()
        ));
    }

    // Checks that the (unprivileged) committee user can read the others' project.
    #[tokio::test]
    async fn test_committee_other() {
        let user = test::model::new_committee_user();
        let other = test::model::new_general_user();
        let project_other = test::model::new_general_project(other.id.clone());

        let app = test::build_mock_app()
            .users(vec![user.clone(), other.clone()])
            .projects(vec![project_other.clone()])
            .build()
            .login_as(user.clone())
            .await;

        assert!(matches!(
            get_project_by_display_id::run(&app, project_other.display_id.into_string()).await,
            Ok(got)
            if got.id == ProjectId::from_entity(project_other.id) && got.name == project_other.name.into_string()
        ));
    }

    // Checks that the privileged committee user can read the others' project.
    #[tokio::test]
    async fn test_operator_other() {
        let user = test::model::new_operator_user();
        let other = test::model::new_general_user();
        let project_other = test::model::new_general_project(other.id.clone());

        let app = test::build_mock_app()
            .users(vec![user.clone(), other.clone()])
            .projects(vec![project_other.clone()])
            .build()
            .login_as(user.clone())
            .await;

        assert!(matches!(
            get_project_by_display_id::run(&app, project_other.display_id.into_string()).await,
            Ok(got)
            if got.id == ProjectId::from_entity(project_other.id) && got.name == project_other.name.into_string()
        ));
    }

    // Checks that the `NotFound` is returned when the unprivileged committee user
    // attempt to read the non-existing others' project.
    #[tokio::test]
    async fn test_committee_nonexisting_other() {
        let user = test::model::new_committee_user();
        let other = test::model::new_general_user();
        let project_other = test::model::new_general_project(other.id.clone());

        let app = test::build_mock_app()
            .users(vec![user.clone(), other.clone()])
            .projects(vec![]) // no projects created
            .build()
            .login_as(user.clone())
            .await;

        assert!(matches!(
            get_project_by_display_id::run(&app, project_other.display_id.into_string()).await,
            Err(UseCaseError::UseCase(
                get_project_by_display_id::Error::NotFound
            ))
        ));
    }
}
