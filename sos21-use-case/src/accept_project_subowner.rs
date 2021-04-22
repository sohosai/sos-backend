use crate::error::{UseCaseError, UseCaseResult};
use crate::model::pending_project::PendingProjectId;
use crate::model::project::{Project, ProjectFromEntityInput};

use anyhow::Context;
use sos21_domain::context::{Login, PendingProjectRepository, ProjectRepository};
use sos21_domain::model::pending_project;

#[derive(Debug, Clone)]
pub enum Error {
    PendingProjectNotFound,
    TooManyProjects,
}

impl Error {
    fn from_accept_error(_err: pending_project::TooManyProjectsError) -> Self {
        Error::TooManyProjects
    }
}

#[tracing::instrument(skip(ctx))]
pub async fn run<C>(
    ctx: &Login<C>,
    pending_project_id: PendingProjectId,
) -> UseCaseResult<Project, Error>
where
    C: ProjectRepository + PendingProjectRepository + Send + Sync,
{
    let login_user = ctx.login_user();

    let result = ctx
        .get_pending_project(pending_project_id.into_entity())
        .await
        .context("Failed to get a pending project")?;
    let result = match result {
        Some(result) if result.pending_project.is_visible_to(login_user) => result,
        _ => return Err(UseCaseError::UseCase(Error::PendingProjectNotFound)),
    };

    ctx.delete_pending_project(result.pending_project.id)
        .await
        .context("Failed to delete a pending project")?;

    let project = result
        .pending_project
        .accept_subowner(&ctx, login_user)
        .await?
        .map_err(|err| UseCaseError::UseCase(Error::from_accept_error(err)))?;

    ctx.store_project(project.clone())
        .await
        .context("Failed to store a project")?;

    use_case_ensure!(project.is_visible_to(login_user));
    Ok(Project::from_entity(ProjectFromEntityInput {
        project,
        owner_name: result.author.name,
        owner_kana_name: result.author.kana_name,
        subowner_name: login_user.name.clone(),
        subowner_kana_name: login_user.kana_name.clone(),
    }))
}

#[cfg(test)]
mod tests {
    use crate::model::pending_project::PendingProjectId;
    use crate::model::user::UserId;
    use crate::{accept_project_subowner, get_pending_project, UseCaseError};
    use sos21_domain::test;

    #[tokio::test]
    async fn test_owner() {
        let user = test::model::new_general_user();
        let pending_project = test::model::new_general_pending_project(user.id.clone());

        let app = test::build_mock_app()
            .users(vec![user.clone()])
            .pending_projects(vec![pending_project.clone()])
            .build()
            .login_as(user.clone())
            .await;

        let pending_project_id = PendingProjectId::from_entity(pending_project.id);

        assert!(matches!(
            accept_project_subowner::run(&app, pending_project_id).await,
            Ok(got)
            if got.owner_id == UserId::from_entity(user.id.clone())
            && got.subowner_id == UserId::from_entity(user.id)
        ));

        assert!(matches!(
            get_pending_project::run(&app, pending_project_id).await,
            Err(UseCaseError::UseCase(get_pending_project::Error::NotFound))
        ));
    }

    #[tokio::test]
    async fn test_other() {
        let user = test::model::new_general_user();
        let other = test::model::new_general_user();
        let pending_project = test::model::new_general_pending_project(other.id.clone());

        let app = test::build_mock_app()
            .users(vec![user.clone(), other.clone()])
            .pending_projects(vec![pending_project.clone()])
            .build()
            .login_as(user.clone())
            .await;

        let pending_project_id = PendingProjectId::from_entity(pending_project.id);

        assert!(matches!(
            accept_project_subowner::run(&app, pending_project_id).await,
            Ok(got)
            if got.owner_id == UserId::from_entity(other.id)
            && got.subowner_id == UserId::from_entity(user.id)
        ));

        assert!(matches!(
            get_pending_project::run(&app, pending_project_id).await,
            Err(UseCaseError::UseCase(get_pending_project::Error::NotFound))
        ));
    }
}
