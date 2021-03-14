use crate::error::{UseCaseError, UseCaseResult};
use crate::model::file::{File, FileId};

use anyhow::Context;
use sos21_domain::context::{FileRepository, Login};

#[derive(Debug, Clone)]
pub enum Error {
    NotFound,
}

#[tracing::instrument(skip(ctx))]
pub async fn run<C>(ctx: &Login<C>, file_id: FileId) -> UseCaseResult<File, Error>
where
    C: FileRepository + Send + Sync,
{
    let login_user = ctx.login_user();

    let result = ctx
        .get_file(file_id.into_entity())
        .await
        .context("Failed to get a file")?;
    let file = match result {
        Some(x) => x,
        None => return Err(UseCaseError::UseCase(Error::NotFound)),
    };

    if !file.is_visible_to(login_user) {
        return Err(UseCaseError::UseCase(Error::NotFound));
    }

    Ok(File::from_entity(file))
}

#[cfg(test)]
mod tests {
    use crate::model::file::FileId;
    use crate::{get_file, UseCaseError};
    use sos21_domain::test;

    // Checks that the normal user can read their file.
    #[tokio::test]
    async fn test_general_owner() {
        let user = test::model::new_general_user();
        let (object, size) = test::model::new_object();
        let file = test::model::new_file(user.id.clone(), object.id, size);

        let app = test::build_mock_app()
            .users(vec![user.clone()])
            .files(vec![file.clone()])
            .objects(vec![object])
            .await
            .build()
            .login_as(user.clone())
            .await;

        let file_id = FileId::from_entity(file.id);
        assert!(matches!(
            get_file::run(&app, file_id).await,
            Ok(file)
            if file.id == file_id
        ));
    }

    // Checks that the normal user cannot read others' file.
    #[tokio::test]
    async fn test_general_other() {
        let user = test::model::new_general_user();
        let other = test::model::new_general_user();
        let (object, size) = test::model::new_object();
        let file_other = test::model::new_file(other.id.clone(), object.id, size);

        let app = test::build_mock_app()
            .users(vec![user.clone(), other.clone()])
            .files(vec![file_other.clone()])
            .objects(vec![object])
            .await
            .build()
            .login_as(user.clone())
            .await;

        assert!(matches!(
            get_file::run(&app, FileId::from_entity(file_other.id)).await,
            Err(UseCaseError::UseCase(get_file::Error::NotFound))
        ));
    }

    // Checks that the comittee user cannot read others' file.
    #[tokio::test]
    async fn test_committee_other() {
        let user = test::model::new_committee_user();
        let other = test::model::new_general_user();
        let (object, size) = test::model::new_object();
        let file_other = test::model::new_file(other.id.clone(), object.id, size);

        let app = test::build_mock_app()
            .users(vec![user.clone(), other.clone()])
            .files(vec![file_other.clone()])
            .objects(vec![object])
            .await
            .build()
            .login_as(user.clone())
            .await;

        assert!(matches!(
            get_file::run(&app, FileId::from_entity(file_other.id)).await,
            Err(UseCaseError::UseCase(get_file::Error::NotFound))
        ));
    }

    // Checks that the privileged comittee user cannot read others' file.
    #[tokio::test]
    async fn test_operator_other() {
        let user = test::model::new_operator_user();
        let other = test::model::new_general_user();
        let (object, size) = test::model::new_object();
        let file_other = test::model::new_file(other.id.clone(), object.id, size);

        let app = test::build_mock_app()
            .users(vec![user.clone(), other.clone()])
            .files(vec![file_other.clone()])
            .objects(vec![object])
            .await
            .build()
            .login_as(user.clone())
            .await;

        assert!(matches!(
            get_file::run(&app, FileId::from_entity(file_other.id)).await,
            Err(UseCaseError::UseCase(get_file::Error::NotFound))
        ));
    }

    // Checks that the admin user cannot read others' file.
    #[tokio::test]
    async fn test_admin_other() {
        let user = test::model::new_admin_user();
        let other = test::model::new_general_user();
        let (object, size) = test::model::new_object();
        let file_other = test::model::new_file(other.id.clone(), object.id, size);

        let app = test::build_mock_app()
            .users(vec![user.clone(), other.clone()])
            .files(vec![file_other.clone()])
            .objects(vec![object])
            .await
            .build()
            .login_as(user.clone())
            .await;

        assert!(matches!(
            get_file::run(&app, FileId::from_entity(file_other.id)).await,
            Err(UseCaseError::UseCase(get_file::Error::NotFound))
        ));
    }
}
