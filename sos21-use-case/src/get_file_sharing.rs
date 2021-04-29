use crate::error::{UseCaseError, UseCaseResult};
use crate::model::file_sharing::{FileSharing, FileSharingId};

use anyhow::Context;
use sos21_domain::context::{FileSharingRepository, Login};

#[derive(Debug, Clone)]
pub enum Error {
    NotFound,
}

#[tracing::instrument(skip(ctx))]
pub async fn run<C>(ctx: &Login<C>, sharing_id: FileSharingId) -> UseCaseResult<FileSharing, Error>
where
    C: FileSharingRepository + Send + Sync,
{
    let login_user = ctx.login_user();

    let result = ctx
        .get_file_sharing(sharing_id.into_entity())
        .await
        .context("Failed to get a file sharing")?;
    let (sharing, file) = match result {
        Some(x) => x,
        None => return Err(UseCaseError::UseCase(Error::NotFound)),
    };

    if !sharing.is_visible_to_with_file(login_user, &file) {
        return Err(UseCaseError::UseCase(Error::NotFound));
    }

    use_case_ensure!(file.is_visible_to(login_user));
    Ok(FileSharing::from_entity(sharing, file))
}

#[cfg(test)]
mod tests {
    use crate::model::file_sharing::FileSharingId;
    use crate::{get_file_sharing, UseCaseError};

    use sos21_domain::model::file_sharing;
    use sos21_domain::test;

    // Checks that the general user can read a file sharing of owning file.
    #[tokio::test]
    async fn test_general_owner() {
        let user = test::model::new_general_user();
        let (file, object) = test::model::new_file(user.id().clone());

        let sharing =
            file_sharing::FileSharing::new(file.id, file_sharing::FileSharingScope::Public);

        let app = test::build_mock_app()
            .users(vec![user.clone()])
            .files(vec![file])
            .objects(vec![object])
            .await
            .sharings(vec![sharing.clone()])
            .build()
            .login_as(user)
            .await;

        let sharing_id = FileSharingId::from_entity(sharing.id());
        assert!(matches!(
            get_file_sharing::run(&app, sharing_id).await,
            Ok(got)
            if got.id == sharing_id
        ));
    }

    // Checks that the general user cannot read a file sharing of other's file.
    #[tokio::test]
    async fn test_general_other() {
        let user = test::model::new_general_user();
        let other = test::model::new_general_user();
        let (other_file, other_object) = test::model::new_file(other.id().clone());

        let sharing =
            file_sharing::FileSharing::new(other_file.id, file_sharing::FileSharingScope::Public);

        let app = test::build_mock_app()
            .users(vec![user.clone(), other])
            .files(vec![other_file])
            .objects(vec![other_object])
            .await
            .sharings(vec![sharing.clone()])
            .build()
            .login_as(user)
            .await;

        let sharing_id = FileSharingId::from_entity(sharing.id());
        assert!(matches!(
            get_file_sharing::run(&app, sharing_id).await,
            Err(UseCaseError::UseCase(get_file_sharing::Error::NotFound))
        ));
    }

    // Checks that the admin user cannot read a file sharing of other's file.
    #[tokio::test]
    async fn test_admin_other() {
        let user = test::model::new_admin_user();
        let other = test::model::new_general_user();
        let (other_file, other_object) = test::model::new_file(other.id().clone());

        let sharing =
            file_sharing::FileSharing::new(other_file.id, file_sharing::FileSharingScope::Public);

        let app = test::build_mock_app()
            .users(vec![user.clone(), other])
            .files(vec![other_file])
            .objects(vec![other_object])
            .await
            .sharings(vec![sharing.clone()])
            .build()
            .login_as(user)
            .await;

        let sharing_id = FileSharingId::from_entity(sharing.id());
        assert!(matches!(
            get_file_sharing::run(&app, sharing_id).await,
            Err(UseCaseError::UseCase(get_file_sharing::Error::NotFound))
        ));
    }
}
