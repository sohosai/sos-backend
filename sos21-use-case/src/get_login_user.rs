use std::convert::Infallible;

use crate::error::UseCaseResult;
use crate::model::user::User;

use sos21_domain_context::Login;

#[tracing::instrument(skip(ctx))]
pub async fn run<C>(ctx: &Login<C>) -> UseCaseResult<User, Infallible> {
    Ok(User::from_entity(ctx.login_user().clone()))
}

#[cfg(test)]
mod tests {
    use crate::get_login_user;
    use crate::model::user::{UserId, UserName};
    use sos21_domain_test as test;

    #[tokio::test]
    async fn test_get() {
        let user = test::model::new_general_user();
        let app = test::build_mock_app()
            .users(vec![user.clone()])
            .build()
            .login_as(user.clone())
            .await;

        assert!(matches!(
            get_login_user::run(&app).await,
            Ok(got)
            if got.id == UserId::from_entity(user.id) && got.name == UserName::from_entity(user.name)
        ));
    }
}
