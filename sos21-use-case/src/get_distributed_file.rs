use crate::error::{UseCaseError, UseCaseResult};
use crate::model::file_distribution::{FileDistributionDistributedFile, FileDistributionId};
use crate::model::project::ProjectId;

use anyhow::Context;
use sos21_domain::context::{FileDistributionRepository, Login, ProjectRepository};
use sos21_domain::model::file_distribution;

#[derive(Debug, Clone)]
pub struct Input {
    pub project_id: ProjectId,
    pub distribution_id: FileDistributionId,
}

#[derive(Debug, Clone)]
pub enum Error {
    ProjectNotFound,
    FileDistributionNotFound,
}

impl Error {
    fn from_not_targeted_error(_err: file_distribution::NotTargetedError) -> Self {
        Error::FileDistributionNotFound
    }
}

#[tracing::instrument(skip(ctx))]
pub async fn run<C>(
    ctx: &Login<C>,
    input: Input,
) -> UseCaseResult<FileDistributionDistributedFile, Error>
where
    C: FileDistributionRepository + ProjectRepository + Send + Sync,
{
    let login_user = ctx.login_user();

    let result = ctx
        .get_project(input.project_id.into_entity())
        .await
        .context("Failed to get a project")?;
    let project = match result {
        Some(result) if result.project.is_visible_to(login_user) => result.project,
        _ => return Err(UseCaseError::UseCase(Error::ProjectNotFound)),
    };

    let result = ctx
        .get_file_distribution(input.distribution_id.into_entity())
        .await
        .context("Failed to get a file distribution")?;
    let distribution = match result {
        Some(distribution) => distribution,
        None => return Err(UseCaseError::UseCase(Error::FileDistributionNotFound)),
    };

    let distributed_file = distribution
        .get_distributed_file_for(&project)
        .map_err(|err| UseCaseError::UseCase(Error::from_not_targeted_error(err)))?;

    use_case_ensure!(distributed_file.is_visible_to_with_project(login_user, &project));
    Ok(FileDistributionDistributedFile::from_entity(
        distributed_file,
    ))
}

#[cfg(test)]
mod tests {
    use crate::model::file_distribution::FileDistributionId;
    use crate::model::file_sharing::FileSharingId;
    use crate::model::project::ProjectId;
    use crate::{get_distributed_file, UseCaseError};

    use sos21_domain::model::file_sharing;
    use sos21_domain::test;

    #[tokio::test]
    async fn test_other_project() {
        let operator = test::model::new_operator_user();
        let (file, object) = test::model::new_file(operator.id().clone());

        let user = test::model::new_general_user();
        let project = test::model::new_general_project(user.id().clone());

        let other = test::model::new_general_user();
        let other_project = test::model::new_general_project(other.id().clone());

        let scope = file_sharing::FileSharingScope::Project(project.id());
        let sharing = file_sharing::FileSharing::new(file.id, scope);
        let files = test::model::mock_file_distribution_files_with_project_sharing(&sharing);
        let distribution =
            test::model::new_file_distribution_with_files(operator.id().clone(), files);

        let app = test::build_mock_app()
            .users(vec![user.clone(), other.clone(), operator.clone()])
            .projects(vec![project.clone(), other_project.clone()])
            .files(vec![file.clone()])
            .objects(vec![object])
            .await
            .sharings(vec![sharing.clone()])
            .distributions(vec![distribution.clone()])
            .build()
            .login_as(user.clone())
            .await;

        let input = get_distributed_file::Input {
            project_id: ProjectId::from_entity(other_project.id()),
            distribution_id: FileDistributionId::from_entity(distribution.id),
        };
        assert!(matches!(
            get_distributed_file::run(&app, input).await,
            Err(UseCaseError::UseCase(
                get_distributed_file::Error::ProjectNotFound
            ))
        ));
    }

    #[tokio::test]
    async fn test_not_targeted() {
        let operator = test::model::new_operator_user();
        let (file, object) = test::model::new_file(operator.id().clone());

        let user = test::model::new_general_user();
        let project = test::model::new_general_project(user.id().clone());

        let other = test::model::new_general_user();
        let other_project = test::model::new_general_project(other.id().clone());

        let scope = file_sharing::FileSharingScope::Project(other_project.id());
        let sharing = file_sharing::FileSharing::new(file.id, scope);
        let files = test::model::mock_file_distribution_files_with_project_sharing(&sharing);
        let distribution =
            test::model::new_file_distribution_with_files(operator.id().clone(), files);

        let app = test::build_mock_app()
            .users(vec![user.clone(), other.clone(), operator.clone()])
            .projects(vec![project.clone(), other_project.clone()])
            .files(vec![file.clone()])
            .objects(vec![object])
            .await
            .sharings(vec![sharing.clone()])
            .distributions(vec![distribution.clone()])
            .build()
            .login_as(user.clone())
            .await;

        let input = get_distributed_file::Input {
            project_id: ProjectId::from_entity(project.id()),
            distribution_id: FileDistributionId::from_entity(distribution.id),
        };
        assert!(matches!(
            get_distributed_file::run(&app, input).await,
            Err(UseCaseError::UseCase(
                get_distributed_file::Error::FileDistributionNotFound
            ))
        ));
    }

    #[tokio::test]
    async fn test_get_owner() {
        let operator = test::model::new_operator_user();
        let (file, object) = test::model::new_file(operator.id().clone());

        let user = test::model::new_general_user();
        let project = test::model::new_general_project(user.id().clone());

        let scope = file_sharing::FileSharingScope::Project(project.id());
        let sharing = file_sharing::FileSharing::new(file.id, scope);
        let files = test::model::mock_file_distribution_files_with_project_sharing(&sharing);
        let distribution =
            test::model::new_file_distribution_with_files(operator.id().clone(), files);

        let app = test::build_mock_app()
            .users(vec![user.clone(), operator.clone()])
            .projects(vec![project.clone()])
            .files(vec![file.clone()])
            .objects(vec![object])
            .await
            .sharings(vec![sharing.clone()])
            .distributions(vec![distribution.clone()])
            .build()
            .login_as(user.clone())
            .await;

        let input = get_distributed_file::Input {
            project_id: ProjectId::from_entity(project.id()),
            distribution_id: FileDistributionId::from_entity(distribution.id),
        };
        assert!(matches!(
            get_distributed_file::run(&app, input).await,
            Ok(got)
            if got.sharing_id == FileSharingId::from_entity(sharing.id())
        ));
    }

    #[tokio::test]
    async fn test_get_subowner() {
        let operator = test::model::new_operator_user();
        let (file, object) = test::model::new_file(operator.id().clone());

        let owner = test::model::new_general_user();
        let user = test::model::new_general_user();
        let project =
            test::model::new_general_project_with_subowner(owner.id().clone(), user.id().clone());

        let scope = file_sharing::FileSharingScope::Project(project.id());
        let sharing = file_sharing::FileSharing::new(file.id, scope);
        let files = test::model::mock_file_distribution_files_with_project_sharing(&sharing);
        let distribution =
            test::model::new_file_distribution_with_files(operator.id().clone(), files);

        let app = test::build_mock_app()
            .users(vec![owner.clone(), user.clone(), operator.clone()])
            .projects(vec![project.clone()])
            .files(vec![file.clone()])
            .objects(vec![object])
            .await
            .sharings(vec![sharing.clone()])
            .distributions(vec![distribution.clone()])
            .build()
            .login_as(user.clone())
            .await;

        let input = get_distributed_file::Input {
            project_id: ProjectId::from_entity(project.id()),
            distribution_id: FileDistributionId::from_entity(distribution.id),
        };
        assert!(matches!(
            get_distributed_file::run(&app, input).await,
            Ok(got)
            if got.sharing_id == FileSharingId::from_entity(sharing.id())
        ));
    }
}
