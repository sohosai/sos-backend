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
    let usage = ctx
        .sum_usage_by_user(login_user.id.clone())
        .await
        .context("Failed to sum file usage")?;

    Ok(Output { usage, quota })
}

#[cfg(test)]
mod tests {
    use crate::get_user_file_usage;
    use sos21_domain::test;

    #[tokio::test]
    async fn test_general() {
        let user = test::model::new_general_user();
        let other = test::model::new_general_user();
        let (object1, size1) = test::model::new_object();
        let file1 = test::model::new_file(user.id.clone(), object1.id, size1);
        let (object2, size2) = test::model::new_object();
        let file2 = test::model::new_file(user.id.clone(), object2.id, size2);
        let (object3, size3) = test::model::new_object();
        let file3 = test::model::new_file(other.id.clone(), object3.id, size3);

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
            if output.usage == size1 + size2
        ));
    }
}
