use crate::error::{UseCaseError, UseCaseResult};
use crate::model::project::{Project, ProjectFromEntityInput};

use anyhow::Context;
use sos21_domain::context::{
    project_repository::{self, ProjectRepository},
    Login,
};
use sos21_domain::model::user;

#[derive(Debug, Clone)]
pub enum Error {
    NotFound,
}

#[tracing::instrument(skip(ctx))]
pub async fn run<C>(ctx: &Login<C>) -> UseCaseResult<Project, Error>
where
    C: ProjectRepository + Send + Sync,
{
    let login_user = ctx.login_user();

    let project_id = match login_user.assignment() {
        Some(user::UserAssignment::ProjectOwner(project_id)) => project_id,
        Some(user::UserAssignment::ProjectSubowner(project_id)) => project_id,
        _ => return Err(UseCaseError::UseCase(Error::NotFound)),
    };

    let project_with_owners = ctx
        .get_project(project_id)
        .await
        .context("Failed to get project")?
        .context("Could not find project referenced by user assignment")?;

    let project_repository::ProjectWithOwners {
        project,
        owner,
        subowner,
    } = project_with_owners;

    use_case_ensure!(
        project.is_visible_to(login_user)
            && owner.name().is_visible_to(login_user)
            && owner.kana_name().is_visible_to(login_user)
            && subowner.name().is_visible_to(login_user)
            && subowner.kana_name().is_visible_to(login_user)
    );

    Ok(Project::from_entity(ProjectFromEntityInput {
        project,
        owner_name: owner.name().clone(),
        owner_kana_name: owner.kana_name().clone(),
        subowner_name: subowner.name().clone(),
        subowner_kana_name: subowner.kana_name().clone(),
    }))
}

#[cfg(test)]
mod tests {
    use crate::model::project::ProjectId;
    use crate::{get_user_project, UseCaseError};
    use sos21_domain::test;

    #[tokio::test]
    async fn test_general_owner() {
        let mut user = test::model::new_general_user();
        let mut other = test::model::new_general_user();
        let project1 = test::model::new_general_project(user.id().clone());
        user.assign_project_owner(&project1).unwrap();
        let project2 = test::model::new_general_project(other.id().clone());
        other.assign_project_owner(&project2).unwrap();

        let app = test::build_mock_app()
            .users(vec![user.clone(), other.clone()])
            .projects(vec![project1.clone(), project2.clone()])
            .build()
            .login_as(user.clone())
            .await;

        let got = get_user_project::run(&app).await.unwrap();
        assert_eq!(got.id, ProjectId::from_entity(project1.id()));
    }

    #[tokio::test]
    async fn test_general_subowner() {
        let mut user = test::model::new_general_user();
        let mut other1 = test::model::new_general_user();
        let mut other2 = test::model::new_general_user();
        let project1 = test::model::new_general_project(other1.id().clone());
        other1.assign_project_owner(&project1).unwrap();
        let project2 =
            test::model::new_general_project_with_subowner(other2.id().clone(), user.id().clone());
        other2.assign_project_owner(&project2).unwrap();
        user.assign_project_subowner(&project2).unwrap();

        let app = test::build_mock_app()
            .users(vec![user.clone(), other1, other2])
            .projects(vec![project1.clone(), project2.clone()])
            .build()
            .login_as(user.clone())
            .await;

        let got = get_user_project::run(&app).await.unwrap();
        assert_eq!(got.id, ProjectId::from_entity(project2.id()));
    }

    #[tokio::test]
    async fn test_general_not_found() {
        let user = test::model::new_general_user();
        let mut other = test::model::new_general_user();
        let project = test::model::new_general_project(other.id().clone());
        other.assign_project_owner(&project).unwrap();

        let app = test::build_mock_app()
            .users(vec![user.clone(), other.clone()])
            .projects(vec![project.clone()])
            .build()
            .login_as(user.clone())
            .await;

        assert!(matches!(
            get_user_project::run(&app).await,
            Err(UseCaseError::UseCase(get_user_project::Error::NotFound))
        ));
    }
}
