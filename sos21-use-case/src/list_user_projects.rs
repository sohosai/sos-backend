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
        .list_projects_by_user(login_user.id.clone())
        .await
        .context("Failed to list projects")?;

    projects
        .into_iter()
        .map(|project_with_owners| {
            let project_repository::ProjectWithOwners {
                project,
                owner,
                subowner,
            } = project_with_owners;

            use_case_ensure!(
                project.is_visible_to(login_user)
                    && owner.name.is_visible_to(login_user)
                    && owner.kana_name.is_visible_to(login_user)
                    && subowner.name.is_visible_to(login_user)
                    && subowner.kana_name.is_visible_to(login_user)
            );

            Ok(Project::from_entity(ProjectFromEntityInput {
                project,
                owner_name: owner.name,
                owner_kana_name: owner.kana_name,
                subowner_name: subowner.name,
                subowner_kana_name: subowner.kana_name,
            }))
        })
        .collect()
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
        let project4 =
            test::model::new_general_project_with_subowner(other.id.clone(), user.id.clone());

        let app = test::build_mock_app()
            .users(vec![user.clone(), other.clone()])
            .projects(vec![
                project1.clone(),
                project2.clone(),
                project3.clone(),
                project4.clone(),
            ])
            .build()
            .login_as(user.clone())
            .await;

        let got = list_user_projects::run(&app).await.unwrap();

        let got: HashSet<_> = got.into_iter().map(|project| project.id).collect();
        let expected: HashSet<_> = vec![project1, project2, project4]
            .into_iter()
            .map(|project| ProjectId::from_entity(project.id()))
            .collect();
        assert_eq!(got, expected);
    }
}
