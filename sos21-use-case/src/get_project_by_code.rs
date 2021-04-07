use crate::error::{UseCaseError, UseCaseResult};
use crate::model::project::{Project, ProjectFromEntityInput};

use anyhow::Context;
use sos21_domain::context::project_repository::{self, ProjectRepository};
use sos21_domain::context::Login;
use sos21_domain::model::project;

#[derive(Debug, Clone)]
pub enum Error {
    InvalidCode,
    NotFound,
}

impl Error {
    fn from_code_error(_err: project::code::ParseCodeError) -> Self {
        Error::InvalidCode
    }
}

#[tracing::instrument(skip(ctx))]
pub async fn run<C>(ctx: &Login<C>, code: String) -> UseCaseResult<Project, Error>
where
    Login<C>: ProjectRepository,
{
    let code = project::ProjectCode::parse(&code)
        .map_err(|err| UseCaseError::UseCase(Error::from_code_error(err)))?;

    let result = ctx
        .get_project_by_index(code.index)
        .await
        .context("Failed to get a project")?;
    let result = match result {
        Some(x) => x,
        None => return Err(UseCaseError::UseCase(Error::NotFound)),
    };
    let project_repository::ProjectWithOwners {
        project,
        owner,
        subowner,
    } = result;

    if project.kind() != code.kind {
        return Err(UseCaseError::UseCase(Error::NotFound));
    }

    let login_user = ctx.login_user();
    if !project.is_visible_to(login_user)
        || !owner.name.is_visible_to(login_user)
        || !owner.kana_name.is_visible_to(login_user)
        || !subowner.name.is_visible_to(login_user)
        || !subowner.kana_name.is_visible_to(login_user)
    {
        return Err(UseCaseError::UseCase(Error::NotFound));
    }

    Ok(Project::from_entity(ProjectFromEntityInput {
        project,
        owner_name: owner.name,
        owner_kana_name: owner.kana_name,
        subowner_name: subowner.name,
        subowner_kana_name: subowner.kana_name,
    }))
}

#[cfg(test)]
mod tests {
    use crate::model::project::ProjectId;
    use crate::{get_project_by_code, UseCaseError};
    use sos21_domain::test;

    // Checks that the normal user cannot read the others' project.
    #[tokio::test]
    async fn test_general_other() {
        let user = test::model::new_general_user();
        let other = test::model::new_general_user();
        let project_other = test::model::new_general_project(other.id.clone());

        let app = test::build_mock_app()
            .users(vec![user.clone(), other.clone()])
            .projects(vec![project_other.clone()])
            .build()
            .login_as(user.clone())
            .await;

        assert!(matches!(
            get_project_by_code::run(&app, project_other.code().to_string()).await,
            Err(UseCaseError::UseCase(get_project_by_code::Error::NotFound))
        ));
    }

    // Checks that the normal user can read the owning project.
    #[tokio::test]
    async fn test_general_owner() {
        let user = test::model::new_general_user();
        let project = test::model::new_general_project(user.id.clone());

        let app = test::build_mock_app()
            .users(vec![user.clone()])
            .projects(vec![project.clone()])
            .build()
            .login_as(user.clone())
            .await;

        assert!(matches!(
            get_project_by_code::run(&app, project.code().to_string()).await,
            Ok(got)
            if got.id == ProjectId::from_entity(project.id) && got.name == project.name.into_string()
        ));
    }

    // Checks that the normal user can read the owning project.
    #[tokio::test]
    async fn test_general_subowner() {
        let owner = test::model::new_general_user();
        let user = test::model::new_general_user();
        let project =
            test::model::new_general_project_with_subowner(owner.id.clone(), user.id.clone());

        let app = test::build_mock_app()
            .users(vec![owner.clone(), user.clone()])
            .projects(vec![project.clone()])
            .build()
            .login_as(user.clone())
            .await;

        assert!(matches!(
            get_project_by_code::run(&app, project.code().to_string()).await,
            Ok(got)
            if got.id == ProjectId::from_entity(project.id) && got.name == project.name.into_string()
        ));
    }

    // Checks that the (unprivileged) committee user can read the others' project.
    #[tokio::test]
    async fn test_committee_other() {
        let user = test::model::new_committee_user();
        let other = test::model::new_general_user();
        let project_other = test::model::new_general_project(other.id.clone());

        let app = test::build_mock_app()
            .users(vec![user.clone(), other.clone()])
            .projects(vec![project_other.clone()])
            .build()
            .login_as(user.clone())
            .await;

        assert!(matches!(
            get_project_by_code::run(&app, project_other.code().to_string()).await,
            Ok(got)
            if got.id == ProjectId::from_entity(project_other.id) && got.name == project_other.name.into_string()
        ));
    }

    // Checks that the privileged committee user can read the others' project.
    #[tokio::test]
    async fn test_operator_other() {
        let user = test::model::new_operator_user();
        let other = test::model::new_general_user();
        let project_other = test::model::new_general_project(other.id.clone());

        let app = test::build_mock_app()
            .users(vec![user.clone(), other.clone()])
            .projects(vec![project_other.clone()])
            .build()
            .login_as(user.clone())
            .await;

        assert!(matches!(
            get_project_by_code::run(&app, project_other.code().to_string()).await,
            Ok(got)
            if got.id == ProjectId::from_entity(project_other.id) && got.name == project_other.name.into_string()
        ));
    }

    // Checks that the `NotFound` is returned when the unprivileged committee user
    // attempt to read the non-existing others' project.
    #[tokio::test]
    async fn test_committee_nonexisting_other() {
        let user = test::model::new_committee_user();
        let other = test::model::new_general_user();
        let project_other = test::model::new_general_project(other.id.clone());

        let app = test::build_mock_app()
            .users(vec![user.clone(), other.clone()])
            .projects(vec![]) // no projects created
            .build()
            .login_as(user.clone())
            .await;

        assert!(matches!(
            get_project_by_code::run(&app, project_other.code().to_string()).await,
            Err(UseCaseError::UseCase(get_project_by_code::Error::NotFound))
        ));
    }

    // Checks that the `NotFound` is returned when the unprivileged committee user
    // attempt to read the existing others' project by that index but the kind is different.
    #[tokio::test]
    async fn test_committee_different_kind_other() {
        let user = test::model::new_committee_user();
        let other = test::model::new_general_user();
        let project_other = test::model::new_general_project(other.id.clone());

        let app = test::build_mock_app()
            .users(vec![user.clone(), other.clone()])
            .projects(vec![project_other.clone()])
            .build()
            .login_as(user.clone())
            .await;

        let mut code = project_other.code();
        code.kind.is_cooking = !code.kind.is_cooking;
        assert!(matches!(
            get_project_by_code::run(&app, code.to_string()).await,
            Err(UseCaseError::UseCase(get_project_by_code::Error::NotFound))
        ));
    }
}
