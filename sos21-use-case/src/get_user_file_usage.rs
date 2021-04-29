use std::convert::Infallible;

use crate::error::UseCaseResult;

use anyhow::Context;
use sos21_domain::context::{FileRepository, Login};

#[derive(Clone, Debug)]
pub struct Output {
    pub usage: u64,
    pub quota: Option<u64>,
}

#[tracing::instrument(skip(ctx))]
pub async fn run<C>(ctx: &Login<C>) -> UseCaseResult<Output, Infallible>
where
    C: FileRepository + Send + Sync,
{
    let login_user = ctx.login_user();

    let quota = login_user.file_usage_quota();
    let usage = login_user
        .file_usage(ctx)
        .await
        .context("Failed to get user's file usage")?;

    Ok(Output {
        usage: usage.to_number_of_bytes(),
        quota: quota.max_number_of_bytes(),
    })
}

#[cfg(test)]
mod tests {
    use crate::get_user_file_usage;
    use sos21_domain::test;

    #[tokio::test]
    async fn test_general() {
        let user = test::model::new_general_user();
        let other = test::model::new_general_user();
        let (file1, object1) = test::model::new_file(user.id().clone());
        let (file2, object2) = test::model::new_file(user.id().clone());
        let (file3, object3) = test::model::new_file(other.id().clone());

        let app = test::build_mock_app()
            .users(vec![user.clone(), other.clone()])
            .files(vec![file1.clone(), file2.clone(), file3.clone()])
            .objects(vec![object1, object2, object3])
            .await
            .build()
            .login_as(user.clone())
            .await;

        assert!(matches!(
            get_user_file_usage::run(&app).await,
            Ok(output)
            if output.usage == file1.size.to_number_of_bytes() + file2.size.to_number_of_bytes()
        ));
    }
}
