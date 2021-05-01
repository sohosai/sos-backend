use crate::error::{UseCaseError, UseCaseResult};
use crate::model::file::File;
use crate::model::file_sharing::FileSharingId;

use anyhow::Context;
use sos21_domain::context::{FileSharingRepository, Login};
use sos21_domain::model::file_sharing;

#[derive(Debug, Clone)]
pub enum Error {
    NotFound,
    InvalidSharing,
}

impl Error {
    fn from_witness_error(err: file_sharing::ToWitnessError) -> Self {
        match err.kind() {
            file_sharing::ToWitnessErrorKind::OutOfScope => Error::NotFound,
            file_sharing::ToWitnessErrorKind::ExpiredSharing => Error::InvalidSharing,
            file_sharing::ToWitnessErrorKind::RevokedSharing => Error::InvalidSharing,
        }
    }
}

#[tracing::instrument(skip(ctx))]
pub async fn run<C>(ctx: &Login<C>, sharing_id: FileSharingId) -> UseCaseResult<File, Error>
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

    let witness = sharing
        .to_witness_with_user(login_user)
        .map_err(|err| UseCaseError::UseCase(Error::from_witness_error(err)))?;

    use_case_ensure!(file.is_visible_to_with_sharing(&witness));
    Ok(File::from_entity(file))
}

#[cfg(test)]
mod tests {
    use crate::model::file::FileId;
    use crate::model::file_sharing::FileSharingId;
    use crate::{get_shared_file, UseCaseError};

    use sos21_domain::context::Login;
    use sos21_domain::model::{file, file_sharing, user};
    use sos21_domain::test::{self, context::MockApp};

    async fn mock_env(
        user_role: user::UserRole,
        share_scope: file_sharing::FileSharingScope,
    ) -> (Login<MockApp>, file::File, file_sharing::FileSharing) {
        let user = test::model::new_user(user_role);
        let other = test::model::new_general_user();
        let (other_file, other_object) = test::model::new_file(other.id().clone());

        let sharing = file_sharing::FileSharing::new(other_file.id, share_scope);

        let app = test::build_mock_app()
            .users(vec![user.clone(), other.clone()])
            .files(vec![other_file.clone()])
            .objects(vec![other_object])
            .await
            .sharings(vec![sharing.clone()])
            .build()
            .login_as(user.clone())
            .await;

        (app, other_file, sharing)
    }

    // Checks that the normal user cannot read others' file which is shared to committee users.
    #[tokio::test]
    async fn test_general_committee() {
        let (app, _, sharing) = mock_env(
            user::UserRole::General,
            file_sharing::FileSharingScope::Committee,
        )
        .await;

        assert!(matches!(
            get_shared_file::run(&app, FileSharingId::from_entity(sharing.id())).await,
            Err(UseCaseError::UseCase(get_shared_file::Error::NotFound))
        ));
    }

    // Checks that the committee user can read others' file which is shared to committee users.
    #[tokio::test]
    async fn test_committee_committee() {
        let (app, other_file, sharing) = mock_env(
            user::UserRole::Committee,
            file_sharing::FileSharingScope::Committee,
        )
        .await;

        assert!(matches!(
            get_shared_file::run(&app, FileSharingId::from_entity(sharing.id())).await,
            Ok(file)
            if file.id == FileId::from_entity(other_file.id)
        ));
    }

    // Checks that the operator user can read others' file which is shared to committee users.
    #[tokio::test]
    async fn test_operator_committee() {
        let (app, other_file, sharing) = mock_env(
            user::UserRole::CommitteeOperator,
            file_sharing::FileSharingScope::Committee,
        )
        .await;

        assert!(matches!(
            get_shared_file::run(&app, FileSharingId::from_entity(sharing.id())).await,
            Ok(file)
            if file.id == FileId::from_entity(other_file.id)
        ));
    }

    // Checks that the committee user cannot read others' file which is shared to operator users.
    #[tokio::test]
    async fn test_committee_operator() {
        let (app, _, sharing) = mock_env(
            user::UserRole::Committee,
            file_sharing::FileSharingScope::CommitteeOperator,
        )
        .await;

        assert!(matches!(
            get_shared_file::run(&app, FileSharingId::from_entity(sharing.id())).await,
            Err(UseCaseError::UseCase(get_shared_file::Error::NotFound))
        ));
    }

    // Checks that the operator user can read others' file which is shared to operator users.
    #[tokio::test]
    async fn test_operator_operator() {
        let (app, other_file, sharing) = mock_env(
            user::UserRole::CommitteeOperator,
            file_sharing::FileSharingScope::CommitteeOperator,
        )
        .await;

        assert!(matches!(
            get_shared_file::run(&app, FileSharingId::from_entity(sharing.id())).await,
            Ok(file)
            if file.id == FileId::from_entity(other_file.id)
        ));
    }

    // Checks that the operator user cannot read using non-role sharing.
    #[tokio::test]
    async fn test_non_role() {
        let (app, _, sharing) = mock_env(
            user::UserRole::CommitteeOperator,
            file_sharing::FileSharingScope::Project(test::model::new_project_id()),
        )
        .await;

        assert!(matches!(
            get_shared_file::run(&app, FileSharingId::from_entity(sharing.id())).await,
            Err(UseCaseError::UseCase(get_shared_file::Error::NotFound))
        ));
    }

    // Checks that the operator user cannot read using revoked sharing.
    #[tokio::test]
    async fn test_revoked() {
        let user = test::model::new_operator_user();
        let other = test::model::new_general_user();
        let (other_file, other_object) = test::model::new_file(other.id().clone());

        let mut sharing = file_sharing::FileSharing::new(
            other_file.id,
            file_sharing::FileSharingScope::Committee,
        );
        sharing.revoke().unwrap();

        let app = test::build_mock_app()
            .users(vec![user.clone(), other.clone()])
            .files(vec![other_file.clone()])
            .objects(vec![other_object])
            .await
            .sharings(vec![sharing.clone()])
            .build()
            .login_as(user.clone())
            .await;

        assert!(matches!(
            get_shared_file::run(&app, FileSharingId::from_entity(sharing.id())).await,
            Err(UseCaseError::UseCase(
                get_shared_file::Error::InvalidSharing
            ))
        ));
    }

    // Checks that the operator user cannot read using expired sharing.
    #[tokio::test]
    async fn test_expired() {
        let user = test::model::new_operator_user();
        let other = test::model::new_general_user();
        let (other_file, other_object) = test::model::new_file(other.id().clone());

        let sharing = test::model::new_expired_file_sharing(
            other_file.id,
            file_sharing::FileSharingScope::Committee,
        );

        let app = test::build_mock_app()
            .users(vec![user.clone(), other.clone()])
            .files(vec![other_file.clone()])
            .objects(vec![other_object])
            .await
            .sharings(vec![sharing.clone()])
            .build()
            .login_as(user.clone())
            .await;

        assert!(matches!(
            get_shared_file::run(&app, FileSharingId::from_entity(sharing.id())).await,
            Err(UseCaseError::UseCase(
                get_shared_file::Error::InvalidSharing
            ))
        ));
    }
}
