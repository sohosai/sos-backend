use crate::error::{UseCaseError, UseCaseResult};
use crate::model::file_distribution::FileDistribution;

use anyhow::Context;
use sos21_domain::context::{FileDistributionRepository, Login};
use sos21_domain::model::{permissions::Permissions, user};

#[derive(Debug, Clone)]
pub enum Error {
    InsufficientPermissions,
}

impl Error {
    fn from_permissions_error(_err: user::RequirePermissionsError) -> Self {
        Error::InsufficientPermissions
    }
}

#[tracing::instrument(skip(ctx))]
pub async fn run<C>(ctx: &Login<C>) -> UseCaseResult<Vec<FileDistribution>, Error>
where
    C: FileDistributionRepository + Send + Sync,
{
    let login_user = ctx.login_user();

    login_user
        .require_permissions(Permissions::READ_ALL_FILE_DISTRIBUTIONS)
        .map_err(|err| UseCaseError::UseCase(Error::from_permissions_error(err)))?;

    let distributions = ctx
        .list_file_distributions()
        .await
        .context("Failed to list file distributions")?;

    let mut result = Vec::new();
    for distribution in distributions {
        use_case_ensure!(distribution.is_visible_to(login_user));
        result.push(FileDistribution::from_entity(distribution));
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use crate::model::file_distribution::FileDistributionId;
    use crate::{list_all_file_distributions, UseCaseError};

    use sos21_domain::model::file_sharing;
    use sos21_domain::test;

    #[tokio::test]
    async fn test_general() {
        let user = test::model::new_general_user();

        let app = test::build_mock_app()
            .users(vec![user.clone()])
            .build()
            .login_as(user)
            .await;

        assert!(matches!(
            list_all_file_distributions::run(&app).await,
            Err(UseCaseError::UseCase(
                list_all_file_distributions::Error::InsufficientPermissions
            ))
        ));
    }

    #[tokio::test]
    async fn test_list() {
        let user = test::model::new_operator_user();

        let other = test::model::new_general_user();
        let other_project = test::model::new_general_online_project(other.id().clone());

        let (file, object) = test::model::new_file(user.id().clone());
        let sharing = file_sharing::FileSharing::new(
            file.id,
            file_sharing::FileSharingScope::Project(other_project.id()),
        );
        let files = test::model::mock_file_distribution_files_with_project_sharing(&sharing);
        let distribution = test::model::new_file_distribution_with_files(user.id().clone(), files);

        let app = test::build_mock_app()
            .users(vec![user.clone(), other.clone()])
            .projects(vec![other_project.clone()])
            .files(vec![file.clone()])
            .objects(vec![object])
            .await
            .sharings(vec![sharing.clone()])
            .distributions(vec![distribution.clone()])
            .build()
            .login_as(user.clone())
            .await;

        assert!(matches!(
            list_all_file_distributions::run(&app).await,
            Ok(distributions)
            if distributions.len() == 1 && distributions[0].id == FileDistributionId::from_entity(distribution.id)
        ));
    }
}
