use std::convert::Infallible;

use crate::error::UseCaseResult;
use crate::model::project::{Project, ProjectFromEntityInput};

use anyhow::Context;
use sos21_domain::context::project_repository::{self, ProjectRepository};
use sos21_domain::context::Login;

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

    let mut result = Vec::new();

    for project_with_owner in projects {
        let project_repository::ProjectWithOwners {
            project,
            owner,
            subowner,
        } = project_with_owner;

        use_case_ensure!(owner.id == login_user.id);
        use_case_ensure!(
            project.is_visible_to(login_user)
                && owner.name.is_visible_to(login_user)
                && owner.kana_name.is_visible_to(login_user)
                && subowner.name.is_visible_to(login_user)
                && subowner.kana_name.is_visible_to(login_user)
        );

        result.push(Project::from_entity(ProjectFromEntityInput {
            project,
            owner_name: owner.name,
            owner_kana_name: owner.kana_name,
            subowner_name: subowner.name,
            subowner_kana_name: subowner.kana_name,
        }));
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use crate::list_user_projects;
    use crate::model::project::ProjectId;
    use sos21_domain::test;

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
