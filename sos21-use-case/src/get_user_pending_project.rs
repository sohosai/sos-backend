use crate::error::{UseCaseError, UseCaseResult};
use crate::model::pending_project::PendingProject;

use anyhow::Context;
use sos21_domain::context::{Login, PendingProjectRepository};
use sos21_domain::model::user;

#[derive(Debug, Clone)]
pub enum Error {
    NotFound,
}

#[tracing::instrument(skip(ctx))]
pub async fn run<C>(ctx: &Login<C>) -> UseCaseResult<PendingProject, Error>
where
    C: PendingProjectRepository + Send + Sync,
{
    let login_user = ctx.login_user();

    let pending_project_id = match login_user.assignment() {
        Some(user::UserAssignment::PendingProjectOwner(pending_project_id)) => pending_project_id,
        _ => return Err(UseCaseError::UseCase(Error::NotFound)),
    };

    let pending_project = ctx
        .get_pending_project(pending_project_id)
        .await
        .context("Failed to get pending project")?
        .context("Could not find pending project referenced by user assignment")?
        .pending_project;

    use_case_ensure!(pending_project.is_visible_to(login_user));
    Ok(PendingProject::from_entity(pending_project))
}

#[cfg(test)]
mod tests {
    use crate::model::pending_project::PendingProjectId;
    use crate::{get_user_pending_project, UseCaseError};

    use sos21_domain::test;

    #[tokio::test]
    async fn test_get() {
        let mut user = test::model::new_general_user();
        let pending_project1 = test::model::new_general_pending_project(user.id.clone());
        user.assign_pending_project_owner(&pending_project1)
            .unwrap();
        let mut other = test::model::new_general_user();
        let pending_project2 = test::model::new_general_pending_project(other.id.clone());
        other
            .assign_pending_project_owner(&pending_project2)
            .unwrap();

        let app = test::build_mock_app()
            .users(vec![user.clone(), other.clone()])
            .pending_projects(vec![pending_project1.clone(), pending_project2.clone()])
            .build()
            .login_as(user.clone())
            .await;

        let got = get_user_pending_project::run(&app).await.unwrap();
        assert_eq!(got.id, PendingProjectId::from_entity(pending_project1.id()));
    }

    #[tokio::test]
    async fn test_not_found() {
        let user = test::model::new_general_user();
        let mut other = test::model::new_general_user();
        let pending_project = test::model::new_general_pending_project(other.id.clone());
        other
            .assign_pending_project_owner(&pending_project)
            .unwrap();

        let app = test::build_mock_app()
            .users(vec![user.clone(), other])
            .pending_projects(vec![pending_project.clone()])
            .build()
            .login_as(user.clone())
            .await;

        assert!(matches!(
            get_user_pending_project::run(&app).await,
            Err(UseCaseError::UseCase(
                get_user_pending_project::Error::NotFound
            ))
        ));
    }
}
