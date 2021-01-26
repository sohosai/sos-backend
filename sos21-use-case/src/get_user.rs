use crate::error::{UseCaseError, UseCaseResult};
use crate::model::user::{User, UserId};

use anyhow::Context;
use sos21_domain_context::{Login, UserRepository};

#[derive(Debug, Clone)]
pub enum Error {
    NotFound,
}

#[tracing::instrument(skip(ctx))]
pub async fn run<C>(ctx: &Login<C>, user_id: UserId) -> UseCaseResult<User, Error>
where
    Login<C>: UserRepository,
{
    let user = ctx
        .get_user(user_id.into_entity())
        .await
        .context("Failed to get a user")?;
    let user = match user {
        Some(x) => x,
        None => return Err(UseCaseError::UseCase(Error::NotFound)),
    };

    if !user.is_visible_to(ctx.login_user()) {
        return Err(UseCaseError::UseCase(Error::NotFound));
    }

    Ok(User::from_entity(user))
}

#[cfg(test)]
mod tests {
    use crate::model::user::UserId;
    use crate::{get_user, UseCaseError};
    use sos21_domain_test as test;

    // Checks that the normal user cannot read the others.
    #[tokio::test]
    async fn test_general_other() {
        let user = test::model::new_general_user();
        let other = test::model::new_general_user();

        let app = test::build_mock_app()
            .users(vec![user.clone(), other.clone()])
            .build()
            .login_as(user.clone())
            .await;

        assert!(matches!(
            get_user::run(&app, UserId::from_entity(other.id)).await,
            Err(UseCaseError::UseCase(get_user::Error::NotFound))
        ));
    }

    // Checks that the (unprivileged) committee user cannot read the others.
    #[tokio::test]
    async fn test_committee_other() {
        let user = test::model::new_committee_user();
        let other = test::model::new_general_user();

        let app = test::build_mock_app()
            .users(vec![user.clone(), other.clone()])
            .build()
            .login_as(user.clone())
            .await;

        assert!(matches!(
            get_user::run(&app, UserId::from_entity(other.id)).await,
            Err(UseCaseError::UseCase(get_user::Error::NotFound))
        ));
    }

    // Checks that the privileged committee user can read the others.
    #[tokio::test]
    async fn test_operator_existing_other() {
        let user = test::model::new_operator_user();
        let other = test::model::new_general_user();

        let app = test::build_mock_app()
            .users(vec![user.clone(), other.clone()])
            .build()
            .login_as(user.clone())
            .await;

        let other_id = UserId::from_entity(other.id);
        assert!(matches!(
            get_user::run(&app, other_id.clone()).await,
            Ok(got)
            if got.id == other_id
        ));
    }

    // Checks that the `NotFound` is returned when the privileged committee user
    // attempt to read the non-existing others.
    #[tokio::test]
    async fn test_operator_nonexisting_other() {
        let user = test::model::new_operator_user();
        let other = test::model::new_general_user();

        let app = test::build_mock_app()
            .users(vec![user.clone()])
            .build()
            .login_as(user.clone())
            .await;

        assert!(matches!(
            get_user::run(&app, UserId::from_entity(other.id)).await,
            Err(UseCaseError::UseCase(get_user::Error::NotFound))
        ));
    }
}
