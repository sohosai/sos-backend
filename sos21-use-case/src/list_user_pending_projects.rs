use std::convert::Infallible;

use crate::error::UseCaseResult;
use crate::model::pending_project::PendingProject;

use anyhow::Context;
use sos21_domain::context::{Login, PendingProjectRepository};

#[tracing::instrument(skip(ctx))]
pub async fn run<C>(ctx: &Login<C>) -> UseCaseResult<Vec<PendingProject>, Infallible>
where
    C: PendingProjectRepository + Send + Sync,
{
    let login_user = ctx.login_user();

    let pending_projects = ctx
        .list_pending_projects_by_user(login_user.id.clone())
        .await
        .context("Failed to list a pending projects")?;

    pending_projects
        .into_iter()
        .map(|pending_project| {
            use_case_ensure!(pending_project.is_visible_to(login_user));
            Ok(PendingProject::from_entity(pending_project))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::list_user_pending_projects;
    use crate::model::pending_project::PendingProjectId;

    use sos21_domain::test;

    #[tokio::test]
    async fn test_get() {
        use std::collections::HashSet;

        let user = test::model::new_general_user();
        let pending_project1 = test::model::new_pending_project(user.id.clone());
        let pending_project2 = test::model::new_pending_project(user.id.clone());
        let other = test::model::new_general_user();
        let pending_project3 = test::model::new_pending_project(other.id.clone());

        let app = test::build_mock_app()
            .users(vec![user.clone(), other.clone()])
            .pending_projects(vec![
                pending_project1.clone(),
                pending_project2.clone(),
                pending_project3.clone(),
            ])
            .build()
            .login_as(user.clone())
            .await;

        let pending_projects = list_user_pending_projects::run(&app).await.unwrap();
        let got: HashSet<_> = pending_projects
            .iter()
            .map(|pending_project| pending_project.id)
            .collect();
        let expected: HashSet<_> = vec![pending_project1.id, pending_project2.id]
            .into_iter()
            .map(PendingProjectId::from_entity)
            .collect();
        assert_eq!(got, expected);
    }
}
