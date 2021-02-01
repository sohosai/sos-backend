use std::convert::Infallible;

use crate::error::UseCaseResult;
use crate::model::project::Project;

use anyhow::Context;
use sos21_domain::context::{Login, ProjectRepository};

#[tracing::instrument(skip(ctx))]
pub async fn run<C>(ctx: &Login<C>) -> UseCaseResult<Vec<Project>, Infallible>
where
    Login<C>: ProjectRepository,
{
    let login_user = ctx.login_user();
    let projects = ctx
        .list_projects_by_owner(login_user.id.clone())
        .await
        .context("Failed to list projects")?;

    use_case_ensure!(projects
        .iter()
        .all(|project| project.is_visible_to(login_user)));

    let projects = projects
        .into_iter()
        .map(|project| {
            Project::from_entity(
                project,
                login_user.name.clone(),
                login_user.kana_name.clone(),
            )
        })
        .collect();
    Ok(projects)
}

#[cfg(test)]
mod tests {
    use crate::list_user_projects;
    use crate::model::project::ProjectId;
    use sos21_domain_test as test;

    #[tokio::test]
    async fn test_general() {
        use std::collections::HashSet;

        let user = test::model::new_general_user();
        let other = test::model::new_general_user();
        let project1 = test::model::new_general_project(user.id.clone());
        let project2 = test::model::new_general_project(user.id.clone());
        let project3 = test::model::new_general_project(other.id.clone());

        let app = test::build_mock_app()
            .users(vec![user.clone(), other.clone()])
            .projects(vec![project1.clone(), project2.clone(), project3.clone()])
            .build()
            .login_as(user.clone())
            .await;

        let result = list_user_projects::run(&app).await;
        assert!(result.is_ok());

        let got: HashSet<_> = result
            .unwrap()
            .into_iter()
            .map(|project| project.id)
            .collect();
        let expected: HashSet<_> = vec![project1, project2]
            .into_iter()
            .map(|project| ProjectId::from_entity(project.id))
            .collect();
        assert_eq!(got, expected);
    }
}
