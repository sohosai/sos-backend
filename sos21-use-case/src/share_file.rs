use crate::error::{UseCaseError, UseCaseResult};
use crate::model::file::FileId;
use crate::model::file_sharing::FileSharing;

use anyhow::Context;
use sos21_domain::context::{FileRepository, FileSharingRepository, Login};
use sos21_domain::model::{date_time::DateTime, file, file_sharing, permissions::Permissions};

#[derive(Debug, Clone)]
pub struct Input {
    pub file_id: FileId,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub scope: InputFileSharingScope,
}

#[derive(Debug, Clone)]
pub enum InputFileSharingScope {
    Committee,
    CommitteeOperator,
    Public,
}

#[derive(Debug, Clone)]
pub enum Error {
    InsufficientPermissions,
    FileNotFound,
    NonSharableFile,
    InvalidExpirationDate,
}

impl Error {
    fn from_share_with_expiration_error(err: file::ShareWithExpirationError) -> Self {
        match err.kind() {
            file::ShareWithExpirationErrorKind::InvalidExpirationDate => {
                Error::InvalidExpirationDate
            }
            file::ShareWithExpirationErrorKind::NonSharableFile => Error::NonSharableFile,
        }
    }

    fn from_share_error(_err: file::NonSharableFileError) -> Self {
        Error::NonSharableFile
    }
}

#[tracing::instrument(skip(ctx))]
pub async fn run<C>(ctx: &Login<C>, input: Input) -> UseCaseResult<FileSharing, Error>
where
    C: FileRepository + FileSharingRepository + Send + Sync,
{
    let login_user = ctx.login_user();

    if login_user
        .require_permissions(Permissions::SHARE_FILES)
        .is_err()
    {
        return Err(UseCaseError::UseCase(Error::InsufficientPermissions));
    }

    let expires_at = input.expires_at.map(DateTime::from_utc);
    let scope = to_file_sharing_scope(input.scope);

    let result = ctx
        .get_file(input.file_id.into_entity())
        .await
        .context("Failed to get a file")?;
    let file = match result {
        Some(file) if file.is_visible_to(login_user) => file,
        _ => return Err(UseCaseError::UseCase(Error::FileNotFound)),
    };

    let sharing = if let Some(expires_at) = expires_at {
        file.share_with_expiration_by(login_user, scope, expires_at)
            .map_err(|err| UseCaseError::UseCase(Error::from_share_with_expiration_error(err)))?
    } else {
        file.share_by(login_user, scope)
            .map_err(|err| UseCaseError::UseCase(Error::from_share_error(err)))?
    };

    ctx.store_file_sharing(sharing.clone())
        .await
        .context("Failed to store a file sharing")?;
    use_case_ensure!(sharing.is_visible_to_with_file(login_user, &file));
    use_case_ensure!(file.is_visible_to(login_user));
    Ok(FileSharing::from_entity(sharing, file))
}

fn to_file_sharing_scope(scope: InputFileSharingScope) -> file_sharing::FileSharingScope {
    match scope {
        InputFileSharingScope::Committee => file_sharing::FileSharingScope::Committee,
        InputFileSharingScope::CommitteeOperator => {
            file_sharing::FileSharingScope::CommitteeOperator
        }
        InputFileSharingScope::Public => file_sharing::FileSharingScope::Public,
    }
}

#[cfg(test)]
mod tests {
    use crate::model::file::FileId;
    use crate::{get_file_sharing, share_file, UseCaseError};
    use sos21_domain::test;

    // Checks that the normal user can share owning file.
    #[tokio::test]
    async fn test_general_owner() {
        let user = test::model::new_general_user();
        let (file, object) = test::model::new_file(user.id.clone());

        let app = test::build_mock_app()
            .users(vec![user.clone()])
            .files(vec![file.clone()])
            .objects(vec![object])
            .await
            .build()
            .login_as(user)
            .await;

        let file_id = FileId::from_entity(file.id);
        let input = share_file::Input {
            file_id,
            expires_at: None,
            scope: share_file::InputFileSharingScope::Public,
        };
        let sharing = share_file::run(&app, input).await.unwrap();
        assert_eq!(sharing.file_id, file_id);

        assert!(matches!(
            get_file_sharing::run(&app, sharing.id).await,
            Ok(got)
            if got.id == sharing.id
        ));
    }

    // Checks that the normal user can share owning file with expiration date.
    #[tokio::test]
    async fn test_general_owner_with_expiration() {
        let user = test::model::new_general_user();
        let (file, object) = test::model::new_file(user.id.clone());

        let app = test::build_mock_app()
            .users(vec![user.clone()])
            .files(vec![file.clone()])
            .objects(vec![object])
            .await
            .build()
            .login_as(user)
            .await;

        let file_id = FileId::from_entity(file.id);
        let input = share_file::Input {
            file_id,
            expires_at: Some(chrono::Utc::now() + chrono::Duration::hours(1)),
            scope: share_file::InputFileSharingScope::Public,
        };
        let sharing = share_file::run(&app, input).await.unwrap();
        assert_eq!(sharing.file_id, file_id);

        assert!(matches!(
            get_file_sharing::run(&app, sharing.id).await,
            Ok(got)
            if got.id == sharing.id
        ));
    }

    // Checks that the normal user cannot share a file with past expiration date.
    #[tokio::test]
    async fn test_past_expiration() {
        let user = test::model::new_general_user();
        let (file, object) = test::model::new_file(user.id.clone());

        let app = test::build_mock_app()
            .users(vec![user.clone()])
            .files(vec![file.clone()])
            .objects(vec![object])
            .await
            .build()
            .login_as(user)
            .await;

        let file_id = FileId::from_entity(file.id);
        let input = share_file::Input {
            file_id,
            expires_at: Some(chrono::Utc::now() - chrono::Duration::hours(1)),
            scope: share_file::InputFileSharingScope::Public,
        };
        assert!(matches!(
            share_file::run(&app, input).await,
            Err(UseCaseError::UseCase(
                share_file::Error::InvalidExpirationDate
            ))
        ));
    }

    // Checks that the normal user cannot share other's file.
    #[tokio::test]
    async fn test_general_other() {
        let user = test::model::new_general_user();
        let other = test::model::new_general_user();
        let (other_file, other_object) = test::model::new_file(other.id.clone());

        let app = test::build_mock_app()
            .users(vec![user.clone(), other])
            .files(vec![other_file.clone()])
            .objects(vec![other_object])
            .await
            .build()
            .login_as(user)
            .await;

        let input = share_file::Input {
            file_id: FileId::from_entity(other_file.id),
            expires_at: None,
            scope: share_file::InputFileSharingScope::Public,
        };
        assert!(matches!(
            share_file::run(&app, input).await,
            Err(UseCaseError::UseCase(share_file::Error::FileNotFound))
        ));
    }

    // Checks that the admin user cannot share other's file.
    #[tokio::test]
    async fn test_admin_other() {
        let user = test::model::new_admin_user();
        let other = test::model::new_general_user();
        let (other_file, other_object) = test::model::new_file(other.id.clone());

        let app = test::build_mock_app()
            .users(vec![user.clone(), other])
            .files(vec![other_file.clone()])
            .objects(vec![other_object])
            .await
            .build()
            .login_as(user)
            .await;

        let input = share_file::Input {
            file_id: FileId::from_entity(other_file.id),
            expires_at: None,
            scope: share_file::InputFileSharingScope::Public,
        };
        assert!(matches!(
            share_file::run(&app, input).await,
            Err(UseCaseError::UseCase(share_file::Error::FileNotFound))
        ));
    }
}
