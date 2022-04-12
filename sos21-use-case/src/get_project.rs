use crate::error::{UseCaseError, UseCaseResult};
use crate::model::project::{Project, ProjectFromEntityInput, ProjectId};

use anyhow::Context;
use sos21_domain::context::{Login, ProjectRepository};

#[derive(Debug, Clone)]
pub enum Error {
    NotFound,
}

#[tracing::instrument(skip(ctx))]
pub async fn run<C>(ctx: &Login<C>, project_id: ProjectId) -> UseCaseResult<Project, Error>
where
    Login<C>: ProjectRepository,
{
    let result = ctx
        .get_project(project_id.into_entity())
        .await
        .context("Failed to get a project")?;
    let result = match result {
        Some(x) => x,
        None => return Err(UseCaseError::UseCase(Error::NotFound)),
    };

    let login_user = ctx.login_user();
    if !result.project.is_visible_to(login_user)
        || !result.owner.name().is_visible_to(login_user)
        || !result.owner.kana_name().is_visible_to(login_user)
        || !result.subowner.name().is_visible_to(login_user)
        || !result.subowner.kana_name().is_visible_to(login_user)
    {
        return Err(UseCaseError::UseCase(Error::NotFound));
    }

    Ok(Project::from_entity(ProjectFromEntityInput {
        project: result.project,
        owner_name: result.owner.name().clone(),
        owner_kana_name: result.owner.kana_name().clone(),
        subowner_name: result.subowner.name().clone(),
        subowner_kana_name: result.subowner.kana_name().clone(),
    }))
}

#[cfg(test)]
mod tests {
    use crate::model::project::ProjectId;
    use crate::{get_project, UseCaseError};
    use sos21_domain::test;

    // Checks that the normal user cannot read the others' project.
    #[tokio::test]
    async fn test_general_other() {
        let user = test::model::new_general_user();
        let other = test::model::new_general_user();
        let project_other = test::model::new_general_online_project(other.id().clone());

        let app = test::build_mock_app()
            .users(vec![user.clone(), other.clone()])
            .projects(vec![project_other.clone()])
            .build()
            .login_as(user.clone())
            .await;

        assert!(matches!(
            get_project::run(&app, ProjectId::from_entity(project_other.id())).await,
            Err(UseCaseError::UseCase(get_project::Error::NotFound))
        ));
    }

    // Checks that the normal user can read the owning project.
    #[tokio::test]
    async fn test_general_owner() {
        let user = test::model::new_general_user();
        let project = test::model::new_general_online_project(user.id().clone());

        let app = test::build_mock_app()
            .users(vec![user.clone()])
            .projects(vec![project.clone()])
            .build()
            .login_as(user.clone())
            .await;

        let project_id = ProjectId::from_entity(project.id());
        assert!(matches!(
            get_project::run(&app, project_id).await,
            Ok(got)
            if got.id == project_id && got.name == project.name().as_str()
        ));
    }

    // Checks that the normal user can read the owning project.
    #[tokio::test]
    async fn test_general_subowner() {
        let owner = test::model::new_general_user();
        let user = test::model::new_general_user();
        let project = test::model::new_general_online_project_with_subowner(
            owner.id().clone(),
            user.id().clone(),
        );

        let app = test::build_mock_app()
            .users(vec![owner.clone(), user.clone()])
            .projects(vec![project.clone()])
            .build()
            .login_as(user.clone())
            .await;

        let project_id = ProjectId::from_entity(project.id());
        assert!(matches!(
            get_project::run(&app, project_id).await,
            Ok(got)
            if got.id == project_id && got.name == project.name().as_str()
        ));
    }

    // Checks that the (unprivileged) committee user can read the others' project.
    #[tokio::test]
    async fn test_committee_other() {
        let user = test::model::new_committee_user();
        let other = test::model::new_general_user();
        let project_other = test::model::new_general_online_project(other.id().clone());

        let app = test::build_mock_app()
            .users(vec![user.clone(), other.clone()])
            .projects(vec![project_other.clone()])
            .build()
            .login_as(user.clone())
            .await;

        let project_id = ProjectId::from_entity(project_other.id());
        assert!(matches!(
            get_project::run(&app, project_id).await,
            Ok(got)
            if got.id == project_id && got.name == project_other.name().as_str()
        ));
    }

    // Checks that the privileged committee user can read the others' project.
    #[tokio::test]
    async fn test_operator_other() {
        let user = test::model::new_operator_user();
        let other = test::model::new_general_user();
        let project_other = test::model::new_general_online_project(other.id().clone());

        let app = test::build_mock_app()
            .users(vec![user.clone(), other.clone()])
            .projects(vec![project_other.clone()])
            .build()
            .login_as(user.clone())
            .await;

        let project_id = ProjectId::from_entity(project_other.id());
        assert!(matches!(
            get_project::run(&app, project_id).await,
            Ok(got)
            if got.id == project_id && got.name == project_other.name().as_str()
        ));
    }

    // Checks that the `NotFound` is returned when the unprivileged committee user
    // attempt to read the non-existing others' project.
    #[tokio::test]
    async fn test_committee_nonexisting_other() {
        let user = test::model::new_committee_user();
        let other = test::model::new_general_user();
        let project_other = test::model::new_general_online_project(other.id().clone());

        let app = test::build_mock_app()
            .users(vec![user.clone(), other.clone()])
            .projects(vec![]) // no projects created
            .build()
            .login_as(user.clone())
            .await;

        assert!(matches!(
            get_project::run(&app, ProjectId::from_entity(project_other.id())).await,
            Err(UseCaseError::UseCase(get_project::Error::NotFound))
        ));
    }
}
