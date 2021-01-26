use crate::error::{UseCaseError, UseCaseResult};
use crate::model::user::{User, UserKanaName, UserName};

use anyhow::Context;
use chrono::Utc;
use sos21_domain_context::{Authentication, UserRepository};
use sos21_domain_model::user;

#[derive(Debug, Clone)]
pub enum Error {
    AlreadySignedUp,
    InvalidUserName,
    InvalidUserKanaName,
}

#[derive(Debug, Clone)]
pub struct Input {
    pub name: UserName,
    pub kana_name: UserKanaName,
}

#[tracing::instrument(skip(ctx))]
pub async fn run<C>(ctx: &Authentication<C>, input: Input) -> UseCaseResult<User, Error>
where
    Authentication<C>: UserRepository,
{
    let id = ctx.authenticated_user();

    if ctx.get_user(id.clone()).await?.is_some() {
        return Err(UseCaseError::UseCase(Error::AlreadySignedUp));
    }

    let name = input
        .name
        .into_entity()
        .ok_or(UseCaseError::UseCase(Error::InvalidUserName))?;
    let kana_name = input
        .kana_name
        .into_entity()
        .ok_or(UseCaseError::UseCase(Error::InvalidUserKanaName))?;

    let user = user::User {
        id,
        name,
        kana_name,
        email: ctx.authenticated_email(),
        created_at: Utc::now(),
        role: user::UserRole::General,
    };
    ctx.create_user(user.clone())
        .await
        .context("Failed to create a user")?;
    Ok(User::from_entity(user))
}

#[cfg(test)]
mod tests {
    use crate::model::user::{UserId, UserKanaName, UserName};
    use crate::{signup, UseCaseError};
    use sos21_domain_test as test;

    fn mock_input() -> signup::Input {
        let name = UserName::from_entity(test::model::mock_user_name());
        let kana_name = UserKanaName::from_entity(test::model::mock_user_kana_name());
        signup::Input { name, kana_name }
    }

    #[tokio::test]
    async fn test_signup() {
        let user_id = "test_user_id".to_string();
        let email = "test@example.com".to_string();

        let app = test::build_mock_app()
            .build()
            .authenticate_as(user_id.clone(), email);

        let input = mock_input();
        assert!(matches!(
            signup::run(&app, input).await,
            Ok(got)
            if got.id == UserId(user_id)
        ));
    }

    // Check that the already registered user cannot sign up.
    #[tokio::test]
    async fn test_duplicate() {
        let user = test::model::new_general_user();

        let app = test::build_mock_app()
            .users(vec![user.clone()])
            .build()
            .authenticate_as(user.id.0, user.email.into_string());

        let input = mock_input();
        assert!(matches!(
            signup::run(&app, input).await,
            Err(UseCaseError::UseCase(signup::Error::AlreadySignedUp))
        ));
    }

    // Check that the same user cannot sign up twice.
    #[tokio::test]
    async fn test_twice() {
        let user_id = "test_user_id".to_string();
        let email = "test@example.com".to_string();

        let app = test::build_mock_app()
            .build()
            .authenticate_as(user_id, email);

        let input = mock_input();
        assert!(matches!(signup::run(&app, input.clone()).await, Ok(_)));
        assert!(matches!(
            signup::run(&app, input).await,
            Err(UseCaseError::UseCase(signup::Error::AlreadySignedUp))
        ));
    }
}
