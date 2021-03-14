use std::fmt::{self, Debug};

use crate::error::{UseCaseError, UseCaseResult};
use crate::model::file::File;

use anyhow::Context;
use bytes::Bytes;
use futures::stream::Stream;
use mime::Mime;
use sos21_domain::context::{FileRepository, Login, ObjectRepository};
use sos21_domain::model::date_time::DateTime;
use sos21_domain::model::permissions::Permissions;
use sos21_domain::model::{file, object, user};
use uuid::Uuid;

pub struct Input<S> {
    pub data: S,
    pub name: Option<String>,
    pub content_type: Option<Mime>,
}

impl<S> Debug for Input<S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Input")
            .field("data", &"..")
            .field("name", &self.name)
            .field("content_type", &self.content_type)
            .finish()
    }
}

#[derive(Debug, Clone)]
pub enum Error {
    InvalidName,
    OutOfUsageQuota,
    InsufficientPermissions,
}

impl Error {
    fn from_name_error(_err: file::name::NameError) -> Self {
        Error::InvalidName
    }

    fn from_store_object_error<C>(_err: C::OutOfLimitSizeError) -> Self
    where
        C: ObjectRepository,
    {
        Error::OutOfUsageQuota
    }

    fn from_permissions_error(_err: user::RequirePermissionsError) -> Self {
        Error::InsufficientPermissions
    }
}

#[tracing::instrument(skip(ctx))]
pub async fn run<C, S>(ctx: &Login<C>, input: Input<S>) -> UseCaseResult<File, Error>
where
    C: FileRepository + ObjectRepository + Send + Sync,
    S: Stream<Item = anyhow::Result<Bytes>> + Send + Sync + 'static,
{
    let login_user = ctx.login_user();

    login_user
        .require_permissions(Permissions::CREATE_FILES)
        .map_err(|err| UseCaseError::UseCase(Error::from_permissions_error(err)))?;

    let name = input
        .name
        .map(file::FileName::from_string)
        .transpose()
        .map_err(|err| UseCaseError::UseCase(Error::from_name_error(err)))?;

    let type_ = input
        .content_type
        .map(file::FileType::from_mime)
        .unwrap_or_default();

    let usage = ctx
        .sum_usage_by_user(login_user.id.clone())
        .await
        .context("Failed to sum usage by user")?;

    let object_id = object::ObjectId::from_uuid(Uuid::new_v4());
    let object = object::Object {
        id: object_id,
        data: object::ObjectData::from_stream(input.data),
    };

    let size = if let Some(limit) = login_user.file_usage_quota() {
        if usage >= limit {
            return Err(UseCaseError::UseCase(Error::OutOfUsageQuota));
        }

        ctx.store_object_with_limit(object, limit - usage)
            .await
            .context("Failed to store an object")?
            .map_err(|err| UseCaseError::UseCase(Error::from_store_object_error::<C>(err)))?
    } else {
        ctx.store_object(object)
            .await
            .context("Failed to store an object")?
    };

    let file = file::File {
        id: file::FileId::from_uuid(Uuid::new_v4()),
        created_at: DateTime::now(),
        author_id: login_user.id.clone(),
        object_id,
        name,
        type_,
        size,
    };

    ctx.store_file(file.clone())
        .await
        .context("Failed to store a file")?;

    use_case_ensure!(file.is_visible_to(login_user));
    Ok(File::from_entity(file))
}

#[cfg(test)]
mod tests {
    use crate::{create_file, get_file, UseCaseError};
    use sos21_domain::test;

    #[tokio::test]
    async fn test_general_create() {
        let user = test::model::new_general_user();
        let app = test::build_mock_app()
            .users(vec![user.clone()])
            .build()
            .login_as(user.clone())
            .await;

        let name = "テストテスト".to_string();
        let input = create_file::Input {
            data: test::model::new_object_data().0.into_stream(),
            name: Some(name.clone()),
            content_type: Some(mime::APPLICATION_OCTET_STREAM),
        };

        let file = create_file::run(&app, input).await.unwrap();
        assert_eq!(file.name.unwrap(), name);

        assert!(matches!(
            get_file::run(&app, file.id).await,
            Ok(got)
            if got.id == file.id
        ));
    }

    #[tokio::test]
    async fn test_general_out_of_quota_loading() {
        let user = test::model::new_general_user();
        let limit = user.file_usage_quota().unwrap();
        // to avoid too much load
        // let object = test::model::new_object_with_size(limit - 50);
        let (object, _) = test::model::new_object();
        let file = test::model::new_file(user.id.clone(), object.id, limit - 50);

        let app = test::build_mock_app()
            .users(vec![user.clone()])
            .files(vec![file])
            .objects(vec![object])
            .await
            .build()
            .login_as(user)
            .await;

        let input = create_file::Input {
            data: test::model::new_object_data_with_size(100).into_stream(),
            name: Some("テストテスト".to_string()),
            content_type: Some(mime::APPLICATION_OCTET_STREAM),
        };

        assert!(matches!(
            create_file::run(&app, input).await,
            Err(UseCaseError::UseCase(create_file::Error::OutOfUsageQuota))
        ));
    }

    #[tokio::test]
    async fn test_general_out_of_quota_stored() {
        let user = test::model::new_general_user();
        let limit = user.file_usage_quota().unwrap();
        // to avoid too much load
        // let object = test::model::new_object_with_size(limit + 10);
        let (object, _) = test::model::new_object();
        let file = test::model::new_file(user.id.clone(), object.id, limit + 10);

        let app = test::build_mock_app()
            .users(vec![user.clone()])
            .files(vec![file])
            .objects(vec![object])
            .await
            .build()
            .login_as(user)
            .await;

        let input = create_file::Input {
            data: test::model::new_object_data_with_size(5).into_stream(),
            name: Some("テストテスト".to_string()),
            content_type: Some(mime::APPLICATION_OCTET_STREAM),
        };

        assert!(matches!(
            create_file::run(&app, input).await,
            Err(UseCaseError::UseCase(create_file::Error::OutOfUsageQuota))
        ));
    }
}
