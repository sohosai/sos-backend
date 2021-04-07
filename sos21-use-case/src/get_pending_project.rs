use crate::error::{UseCaseError, UseCaseResult};
use crate::model::pending_project::{PendingProject, PendingProjectId};

use anyhow::Context;
use sos21_domain::context::{Login, PendingProjectRepository};

#[derive(Debug, Clone)]
pub enum Error {
    NotFound,
}

#[tracing::instrument(skip(ctx))]
pub async fn run<C>(
    ctx: &Login<C>,
    pending_project_id: PendingProjectId,
) -> UseCaseResult<PendingProject, Error>
where
    C: PendingProjectRepository + Send + Sync,
{
    let login_user = ctx.login_user();

    let result = ctx
        .get_pending_project(pending_project_id.into_entity())
        .await
        .context("Failed to get a pending project")?;
    let pending_project = match result {
        Some(result) if result.pending_project.is_visible_to(login_user) => result.pending_project,
        _ => return Err(UseCaseError::UseCase(Error::NotFound)),
    };

    Ok(PendingProject::from_entity(pending_project))
}

#[cfg(test)]
mod tests {
    use crate::model::pending_project::PendingProjectId;
    use crate::{get_pending_project, UseCaseError};

    use sos21_domain::test;

    #[tokio::test]
    async fn test_get_owner() {
        let user = test::model::new_general_user();
        let pending_project = test::model::new_pending_project(user.id.clone());

        let app = test::build_mock_app()
            .users(vec![user.clone()])
            .pending_projects(vec![pending_project.clone()])
            .build()
            .login_as(user.clone())
            .await;

        let pending_project_id = PendingProjectId::from_entity(pending_project.id);
        assert!(matches!(
            get_pending_project::run(&app, pending_project_id).await,
            Ok(got)
            if got.id == pending_project_id
        ));
    }

    #[tokio::test]
    async fn test_get_other() {
        let user = test::model::new_general_user();
        let other = test::model::new_general_user();
        let pending_project = test::model::new_pending_project(other.id.clone());

        let app = test::build_mock_app()
            .users(vec![user.clone(), other.clone()])
            .pending_projects(vec![pending_project.clone()])
            .build()
            .login_as(user.clone())
            .await;

        let pending_project_id = PendingProjectId::from_entity(pending_project.id);
        assert!(matches!(
            get_pending_project::run(&app, pending_project_id).await,
            Ok(got)
            if got.id == pending_project_id
        ));
    }

    #[tokio::test]
    async fn test_not_found() {
        let user = test::model::new_general_user();
        let pending_project = test::model::new_pending_project(user.id.clone());

        let app = test::build_mock_app()
            .users(vec![user.clone()])
            .build()
            .login_as(user.clone())
            .await;

        assert!(matches!(
            get_pending_project::run(&app, PendingProjectId::from_entity(pending_project.id)).await,
            Err(UseCaseError::UseCase(get_pending_project::Error::NotFound))
        ));
    }
}
