use crate::error::{UseCaseError, UseCaseResult};
use crate::model::project::Project;

use anyhow::Context;
use sos21_domain::context::{Login, ProjectRepository};
use sos21_domain::model::permissions::Permissions;

#[derive(Debug, Clone)]
pub enum Error {
    InsufficientPermissions,
}

#[tracing::instrument(skip(ctx))]
pub async fn run<C>(ctx: &Login<C>) -> UseCaseResult<Vec<Project>, Error>
where
    Login<C>: ProjectRepository,
{
    let login_user = ctx.login_user();

    if login_user
        .require_permissions(Permissions::READ_ALL_PROJECTS)
        .is_err()
    {
        return Err(UseCaseError::UseCase(Error::InsufficientPermissions));
    }

    let projects = ctx
        .list_projects()
        .await
        .context("Failed to list projects")?;
    use_case_ensure!(projects
        .iter()
        .all(|(project, owner)| project.is_visible_to(login_user)
            && owner.name.is_visible_to(login_user)
            && owner.kana_name.is_visible_to(login_user)));

    let projects = projects
        .into_iter()
        .map(|(project, owner)| Project::from_entity(project, owner.name, owner.kana_name))
        .collect();
    Ok(projects)
}

#[cfg(test)]
mod tests {
    use crate::model::project::ProjectId;
    use crate::{list_all_projects, UseCaseError};
    use sos21_domain::context::Login;
    use sos21_domain::model as domain;
    use sos21_domain::test;

    async fn prepare_app(
        login_user: domain::user::User,
    ) -> (Login<test::context::MockApp>, Vec<domain::project::Project>) {
        let other = test::model::new_general_user();
        let project1 = test::model::new_general_project(login_user.id.clone());
        let project2 = test::model::new_general_project(other.id.clone());

        let app = test::build_mock_app()
            .users(vec![login_user.clone(), other.clone()])
            .projects(vec![project1.clone(), project2.clone()])
            .build()
            .login_as(login_user.clone())
            .await;
        (app, vec![project1, project2])
    }

    // Checks that the normal user cannot list projects.
    #[tokio::test]
    async fn test_general() {
        let user = test::model::new_general_user();
        let (app, _) = prepare_app(user).await;

        assert!(matches!(
            list_all_projects::run(&app).await,
            Err(UseCaseError::UseCase(
                list_all_projects::Error::InsufficientPermissions
            ))
        ));
    }

    // Checks that the (unprivileged) committee user can list projects.
    #[tokio::test]
    async fn test_committee() {
        use std::collections::HashSet;

        let user = test::model::new_committee_user();
        let (app, projects) = prepare_app(user).await;

        let result = list_all_projects::run(&app).await;
        assert!(result.is_ok());

        let got: HashSet<_> = result
            .unwrap()
            .into_iter()
            .map(|project| project.id)
            .collect();
        let expected: HashSet<_> = projects
            .into_iter()
            .map(|project| ProjectId::from_entity(project.id))
            .collect();
        assert_eq!(got, expected);
    }

    // Checks that the privileged committee user can list projects.
    #[tokio::test]
    async fn test_operator() {
        use std::collections::HashSet;

        let user = test::model::new_operator_user();
        let (app, projects) = prepare_app(user).await;

        let result = list_all_projects::run(&app).await;
        assert!(result.is_ok());

        let got: HashSet<_> = result
            .unwrap()
            .into_iter()
            .map(|project| project.id)
            .collect();
        let expected: HashSet<_> = projects
            .into_iter()
            .map(|project| ProjectId::from_entity(project.id))
            .collect();
        assert_eq!(got, expected);
    }
}
