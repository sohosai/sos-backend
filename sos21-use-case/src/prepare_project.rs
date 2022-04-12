use crate::error::{UseCaseError, UseCaseResult};
use crate::model::pending_project::PendingProject;
use crate::model::project::{ProjectAttribute, ProjectCategory};

use anyhow::Context;
use sos21_domain::context::{ConfigContext, Login, PendingProjectRepository, UserRepository};
use sos21_domain::model::{pending_project, project};

#[derive(Debug, Clone)]
pub struct Input {
    pub name: String,
    pub kana_name: String,
    pub group_name: String,
    pub kana_group_name: String,
    pub description: String,
    pub category: ProjectCategory,
    pub attributes: Vec<ProjectAttribute>,
}

#[derive(Debug, Clone)]
pub enum Error {
    InvalidName,
    InvalidKanaName,
    InvalidGroupName,
    InvalidKanaGroupName,
    InvalidDescription,
    DuplicatedAttributes,
    AlreadyProjectOwner,
    AlreadyProjectSubowner,
    AlreadyPendingProjectOwner,
    OutOfCreationPeriod,
    ArtisticStageProject,
}

impl Error {
    fn from_name_error(_err: project::name::NameError) -> Self {
        Error::InvalidName
    }

    fn from_kana_name_error(_err: project::name::KanaNameError) -> Self {
        Error::InvalidKanaName
    }

    fn from_group_name_error(_err: project::name::GroupNameError) -> Self {
        Error::InvalidGroupName
    }

    fn from_kana_group_name_error(_err: project::name::KanaGroupNameError) -> Self {
        Error::InvalidKanaGroupName
    }

    fn from_description_error(_err: project::description::DescriptionError) -> Self {
        Error::InvalidDescription
    }

    fn from_attributes_error(_err: project::attribute::DuplicatedAttributesError) -> Self {
        Error::DuplicatedAttributes
    }

    fn from_new_pending_project_error(err: pending_project::NewPendingProjectError) -> Self {
        match err.kind() {
            pending_project::NewPendingProjectErrorKind::AlreadyProjectOwnerOwner => {
                Error::AlreadyProjectOwner
            }
            pending_project::NewPendingProjectErrorKind::AlreadyProjectSubownerOwner => {
                Error::AlreadyProjectSubowner
            }
            pending_project::NewPendingProjectErrorKind::AlreadyPendingProjectOwnerOwner => {
                Error::AlreadyPendingProjectOwner
            }
            pending_project::NewPendingProjectErrorKind::OutOfCreationPeriod => {
                Error::OutOfCreationPeriod
            }
            pending_project::NewPendingProjectErrorKind::ArtisticStageProject => {
                Error::ArtisticStageProject
            }
        }
    }
}

#[tracing::instrument(skip(ctx))]
pub async fn run<C>(ctx: &Login<C>, input: Input) -> UseCaseResult<PendingProject, Error>
where
    C: PendingProjectRepository + UserRepository + ConfigContext + Send + Sync,
{
    let mut login_user = ctx.login_user().clone();

    let name = project::ProjectName::from_string(input.name)
        .map_err(|err| UseCaseError::UseCase(Error::from_name_error(err)))?;
    let kana_name = project::ProjectKanaName::from_string(input.kana_name)
        .map_err(|err| UseCaseError::UseCase(Error::from_kana_name_error(err)))?;
    let group_name = project::ProjectGroupName::from_string(input.group_name)
        .map_err(|err| UseCaseError::UseCase(Error::from_group_name_error(err)))?;
    let kana_group_name = project::ProjectKanaGroupName::from_string(input.kana_group_name)
        .map_err(|err| UseCaseError::UseCase(Error::from_kana_group_name_error(err)))?;
    let description = project::ProjectDescription::from_string(input.description)
        .map_err(|err| UseCaseError::UseCase(Error::from_description_error(err)))?;

    let attributes = input
        .attributes
        .into_iter()
        .map(ProjectAttribute::into_entity);
    let attributes = project::ProjectAttributes::from_attributes(attributes)
        .map_err(|err| UseCaseError::UseCase(Error::from_attributes_error(err)))?;

    let pending_project = pending_project::PendingProject::new(
        ctx,
        &login_user,
        name,
        kana_name,
        group_name,
        kana_group_name,
        description,
        input.category.into_entity(),
        attributes,
    )
    .map_err(|err| UseCaseError::UseCase(Error::from_new_pending_project_error(err)))?;
    ctx.store_pending_project(pending_project.clone())
        .await
        .context("Failed to create a pending project")?;

    login_user.assign_pending_project_owner(&pending_project)?;
    ctx.store_user(login_user.clone())
        .await
        .context("Failed to create a user")?;

    use_case_ensure!(pending_project.is_visible_to(&login_user));
    Ok(PendingProject::from_entity(pending_project))
}

#[cfg(test)]
mod tests {
    use crate::model::{project::ProjectCategory, user::UserId};
    use crate::{get_pending_project, prepare_project, UseCaseError};
    use sos21_domain::{model::project, test};

    fn mock_input() -> (String, prepare_project::Input) {
        let name = "テストテスト".to_string();
        let input = prepare_project::Input {
            name: name.clone(),
            kana_name: test::model::mock_project_kana_name().into_string(),
            group_name: test::model::mock_project_group_name().into_string(),
            kana_group_name: test::model::mock_project_kana_group_name().into_string(),
            description: test::model::mock_project_description().into_string(),
            category: ProjectCategory::General,
            attributes: Vec::new(),
        };
        (name, input)
    }

    #[tokio::test]
    async fn test_create_without_period() {
        let user = test::model::new_general_user();
        let app = test::build_mock_app()
            .users(vec![user.clone()])
            .build()
            .login_as(user.clone())
            .await;

        let (name, input) = mock_input();
        let pending_project = prepare_project::run(&app, input).await.unwrap();
        assert!(pending_project.name == name);
        assert!(pending_project.owner_id == UserId::from_entity(user.id().clone()));

        assert!(matches!(
            get_pending_project::run(&app, pending_project.id).await,
            Ok(got)
            if got.name == name
        ));
    }

    #[tokio::test]
    async fn test_create_with_period() {
        let user = test::model::new_general_user();
        let period = test::model::new_project_creation_period_from_now();
        let app = test::build_mock_app()
            .users(vec![user.clone()])
            .project_creation_period_for(project::ProjectCategory::General, period)
            .build()
            .login_as(user.clone())
            .await;

        let (name, input) = mock_input();
        let pending_project = prepare_project::run(&app, input).await.unwrap();
        assert!(pending_project.name == name);
        assert!(pending_project.owner_id == UserId::from_entity(user.id().clone()));

        assert!(matches!(
            get_pending_project::run(&app, pending_project.id).await,
            Ok(got)
            if got.name == name
        ));
    }

    #[tokio::test]
    async fn test_create_before_period() {
        let user = test::model::new_general_user();
        let period = test::model::new_project_creation_period_with_hours_from_now(1);
        let app = test::build_mock_app()
            .users(vec![user.clone()])
            .project_creation_period_for(project::ProjectCategory::General, period)
            .build()
            .login_as(user.clone())
            .await;

        let (_, input) = mock_input();
        assert!(matches!(
            prepare_project::run(&app, input).await,
            Err(UseCaseError::UseCase(
                prepare_project::Error::OutOfCreationPeriod
            ))
        ));
    }

    #[tokio::test]
    async fn test_create_after_period() {
        let user = test::model::new_general_user();
        let period = test::model::new_project_creation_period_to_now();
        let app = test::build_mock_app()
            .users(vec![user.clone()])
            .project_creation_period_for(project::ProjectCategory::General, period)
            .build()
            .login_as(user.clone())
            .await;

        let (_, input) = mock_input();
        assert!(matches!(
            prepare_project::run(&app, input).await,
            Err(UseCaseError::UseCase(
                prepare_project::Error::OutOfCreationPeriod
            ))
        ));
    }

    #[tokio::test]
    async fn test_already_project_owner() {
        let mut user = test::model::new_general_user();
        let project = test::model::new_general_project(user.id().clone());
        user.assign_project_owner(&project).unwrap();

        let app = test::build_mock_app()
            .users(vec![user.clone()])
            .projects(vec![project])
            .build()
            .login_as(user.clone())
            .await;

        let (_, input) = mock_input();
        assert!(matches!(
            prepare_project::run(&app, input).await,
            Err(UseCaseError::UseCase(
                prepare_project::Error::AlreadyProjectOwner
            ))
        ));
    }
}
