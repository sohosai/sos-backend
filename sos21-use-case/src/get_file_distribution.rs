use crate::error::{UseCaseError, UseCaseResult};
use crate::model::file_distribution::{FileDistribution, FileDistributionId};

use anyhow::Context;
use sos21_domain::context::{FileDistributionRepository, Login};
use sos21_domain::model::{permissions::Permissions, user};

#[derive(Debug, Clone)]
pub enum Error {
    InsufficientPermissions,
    NotFound,
}

impl Error {
    fn from_permissions_error(_err: user::RequirePermissionsError) -> Self {
        Error::InsufficientPermissions
    }
}

#[tracing::instrument(skip(ctx))]
pub async fn run<C>(
    ctx: &Login<C>,
    distribution_id: FileDistributionId,
) -> UseCaseResult<FileDistribution, Error>
where
    C: FileDistributionRepository + Send + Sync,
{
    let login_user = ctx.login_user();

    login_user
        .require_permissions(Permissions::READ_ALL_FILE_DISTRIBUTIONS)
        .map_err(|err| UseCaseError::UseCase(Error::from_permissions_error(err)))?;

    let result = ctx
        .get_file_distribution(distribution_id.into_entity())
        .await
        .context("Failed to get a file distribution")?;
    let distribution = match result {
        Some(distribution) if distribution.is_visible_to(login_user) => distribution,
        _ => return Err(UseCaseError::UseCase(Error::NotFound)),
    };

    Ok(FileDistribution::from_entity(distribution))
}

#[cfg(test)]
mod tests {
    use crate::model::file_distribution::FileDistributionId;
    use crate::{get_file_distribution, UseCaseError};

    use sos21_domain::model::file_sharing;
    use sos21_domain::test;

    #[tokio::test]
    async fn test_general() {
        let user = test::model::new_general_user();
        let operator = test::model::new_operator_user();

        let other = test::model::new_general_user();
        let other_project = test::model::new_general_project(other.id.clone());

        let (file, object) = test::model::new_file(operator.id.clone());
        let sharing = file_sharing::FileSharing::new(
            file.id,
            file_sharing::FileSharingScope::Project(other_project.id),
        );
        let files = test::model::mock_file_distribution_files_with_project_sharing(&sharing);
        let distribution =
            test::model::new_file_distribution_with_files(operator.id.clone(), files);

        let app = test::build_mock_app()
            .users(vec![user.clone(), other.clone(), operator.clone()])
            .projects(vec![other_project.clone()])
            .files(vec![file.clone()])
            .objects(vec![object])
            .await
            .sharings(vec![sharing.clone()])
            .distributions(vec![distribution.clone()])
            .build()
            .login_as(user.clone())
            .await;

        let distribution_id = FileDistributionId::from_entity(distribution.id);
        assert!(matches!(
            get_file_distribution::run(&app, distribution_id).await,
            Err(UseCaseError::UseCase(
                get_file_distribution::Error::InsufficientPermissions
            ))
        ));
    }

    #[tokio::test]
    async fn test_get() {
        let user = test::model::new_committee_user();
        let operator = test::model::new_operator_user();

        let other = test::model::new_general_user();
        let other_project = test::model::new_general_project(other.id.clone());

        let (file, object) = test::model::new_file(operator.id.clone());
        let sharing = file_sharing::FileSharing::new(
            file.id,
            file_sharing::FileSharingScope::Project(other_project.id),
        );
        let files = test::model::mock_file_distribution_files_with_project_sharing(&sharing);
        let distribution =
            test::model::new_file_distribution_with_files(operator.id.clone(), files);

        let app = test::build_mock_app()
            .users(vec![user.clone(), other.clone(), operator.clone()])
            .projects(vec![other_project.clone()])
            .files(vec![file.clone()])
            .objects(vec![object])
            .await
            .sharings(vec![sharing.clone()])
            .distributions(vec![distribution.clone()])
            .build()
            .login_as(user.clone())
            .await;

        let distribution_id = FileDistributionId::from_entity(distribution.id);
        assert!(matches!(
            get_file_distribution::run(&app, distribution_id).await,
            Ok(got)
            if got.id == distribution_id
        ));
    }
}
