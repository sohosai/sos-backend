use std::convert::Infallible;

use crate::error::UseCaseResult;
use crate::model::file::File;

use anyhow::Context;
use sos21_domain::context::{FileRepository, Login};

#[tracing::instrument(skip(ctx))]
pub async fn run<C>(ctx: &Login<C>) -> UseCaseResult<Vec<File>, Infallible>
where
    C: FileRepository + Send + Sync,
{
    let login_user = ctx.login_user();

    let files = ctx
        .list_files_by_user(login_user.id.clone())
        .await
        .context("Failed to list files")?;

    use_case_ensure!(files.iter().all(|file| file.is_visible_to(login_user)));
    Ok(files.into_iter().map(File::from_entity).collect())
}

#[cfg(test)]
mod tests {
    use crate::list_user_files;
    use crate::model::file::FileId;
    use sos21_domain::test;

    #[tokio::test]
    async fn test_general() {
        use std::collections::HashSet;

        let user = test::model::new_general_user();
        let other = test::model::new_general_user();
        let (file1, object1) = test::model::new_file(user.id.clone());
        let (file2, object2) = test::model::new_file(user.id.clone());
        let (file3, object3) = test::model::new_file(other.id.clone());

        let app = test::build_mock_app()
            .users(vec![user.clone(), other.clone()])
            .files(vec![file1.clone(), file2.clone(), file3.clone()])
            .objects(vec![object1, object2, object3])
            .await
            .build()
            .login_as(user.clone())
            .await;

        let result = list_user_files::run(&app).await;
        assert!(result.is_ok());

        let got: HashSet<_> = result.unwrap().into_iter().map(|file| file.id).collect();
        let expected: HashSet<_> =
            (&[FileId::from_entity(file1.id), FileId::from_entity(file2.id)])
                .iter()
                .cloned()
                .collect();
        assert_eq!(got, expected);
    }
}
