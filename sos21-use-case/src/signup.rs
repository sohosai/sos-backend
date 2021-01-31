use crate::error::{UseCaseError, UseCaseResult};
use crate::model::user::{User, UserKanaName, UserName};

use anyhow::Context;
use chrono::Utc;
use sos21_domain_context::{Authentication, UserRepository};
use sos21_domain_model::{phone_number, user};

#[derive(Debug, Clone)]
pub enum Error {
    AlreadySignedUp,
    InvalidUserName,
    InvalidUserKanaName,
    InvalidPhoneNumber,
    InvalidUserAffiliation,
}

#[derive(Debug, Clone)]
pub struct Input {
    pub name: UserName,
    pub kana_name: UserKanaName,
    pub phone_number: String,
    pub affiliation: String,
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
    let phone_number = phone_number::PhoneNumber::from_string(input.phone_number)
        .map_err(|_| UseCaseError::UseCase(Error::InvalidPhoneNumber))?;
    let affiliation = user::UserAffiliation::from_string(input.affiliation)
        .map_err(|_| UseCaseError::UseCase(Error::InvalidUserAffiliation))?;

    let user = user::User {
        id,
        name,
        kana_name,
        email: ctx.authenticated_email(),
        phone_number,
        affiliation,
        created_at: Utc::now(),
        role: user::UserRole::General,
    };
    ctx.store_user(user.clone())
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
        let phone_number = test::model::mock_phone_number().into_string();
        let affiliation = test::model::mock_user_affiliation().into_string();
        signup::Input {
            name,
            kana_name,
            phone_number,
            affiliation,
        }
    }

    #[tokio::test]
    async fn test_signup() {
        let user_id = "test_user_id".to_string();
        let email = "test@s.tsukuba.ac.jp".to_string();

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
        let email = "test@s.tsukuba.ac.jp".to_string();

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
