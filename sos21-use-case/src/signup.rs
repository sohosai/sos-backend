use crate::error::{UseCaseError, UseCaseResult};
use crate::model::user::{User, UserCategory, UserKanaName, UserName};

use anyhow::Context;
use sos21_domain::context::{
    Authentication, ConfigContext, UserInvitationRepository, UserRepository,
};
use sos21_domain::model::{phone_number, user};

#[derive(Debug, Clone)]
pub enum Error {
    AlreadySignedUp,
    InvalidName,
    InvalidKanaName,
    InvalidPhoneNumber,
}

impl Error {
    fn from_new_user_error(_err: user::AlreadySignedUpError) -> Self {
        Error::AlreadySignedUp
    }

    fn from_name_error(_err: user::name::NameError) -> Self {
        Error::InvalidName
    }

    fn from_kana_name_error(_err: user::name::KanaNameError) -> Self {
        Error::InvalidKanaName
    }

    fn from_phone_number_error(_err: phone_number::FromStringError) -> Self {
        Error::InvalidPhoneNumber
    }
}

#[derive(Debug, Clone)]
pub struct Input {
    pub name: UserName,
    pub kana_name: UserKanaName,
    pub phone_number: String,
    pub category: UserCategory,
}

#[tracing::instrument(skip(ctx))]
pub async fn run<C>(ctx: &Authentication<C>, input: Input) -> UseCaseResult<User, Error>
where
    C: UserRepository + UserInvitationRepository + ConfigContext + Send + Sync,
{
    let id = ctx.authenticated_user();

    let name = user::UserName::from_string(input.name.first, input.name.last)
        .map_err(|err| UseCaseError::UseCase(Error::from_name_error(err)))?;
    let kana_name = user::UserKanaName::from_string(input.kana_name.first, input.kana_name.last)
        .map_err(|err| UseCaseError::UseCase(Error::from_kana_name_error(err)))?;
    let phone_number = phone_number::PhoneNumber::from_string(input.phone_number)
        .map_err(|err| UseCaseError::UseCase(Error::from_phone_number_error(err)))?;
    let category = match input.category {
        UserCategory::UndergraduateStudent=>user::UserCategory::UndergraduateStudent,
        UserCategory::GraduateStudent => user::UserCategory::GraduateStudent,
        UserCategory::AcademicStaff => user::UserCategory::AcademicStaff,
    };

    let user = user::User::new(
        ctx,
        id,
        name,
        kana_name,
        phone_number,
        ctx.authenticated_email(),
        category,
    )
    .await
    .map_err(|err| UseCaseError::from_domain(err, Error::from_new_user_error))?;

    ctx.store_user(user.clone())
        .await
        .context("Failed to create a user")?;
    Ok(User::from_entity(user))
}

#[cfg(test)]
mod tests {
    use crate::model::user::{UserCategory, UserId, UserKanaName, UserName, UserRole};
    use crate::{signup, UseCaseError};
    use sos21_domain::test;

    fn mock_input() -> signup::Input {
        let name = UserName::from_entity(test::model::mock_user_name());
        let kana_name = UserKanaName::from_entity(test::model::mock_user_kana_name());
        let phone_number = test::model::mock_phone_number().into_string();
        let category = UserCategory::from_entity(test::model::mock_user_category());
        signup::Input {
            name,
            kana_name,
            phone_number,
            category,
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
            && got.role == UserRole::General
        ));
    }

    // Check that the already registered user cannot sign up.
    #[tokio::test]
    async fn test_duplicate() {
        let user = test::model::new_general_user();

        let app = test::build_mock_app()
            .users(vec![user.clone()])
            .build()
            .authenticate_as(user.id().clone().0, user.email().clone().into_string());

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

    #[tokio::test]
    async fn test_invitation() {
        let admin = test::model::new_admin_user();
        let user_id = "test_user_id".to_string();
        let email = "test@s.tsukuba.ac.jp".to_string();
        let invitation =
            test::model::new_operator_user_invitation(admin.id().clone(), email.clone());

        let app = test::build_mock_app()
            .users(vec![admin])
            .user_invitations(vec![invitation])
            .build()
            .authenticate_as(user_id.clone(), email);

        let input = mock_input();
        assert!(matches!(
            signup::run(&app, input).await,
            Ok(got)
            if got.id == UserId(user_id)
            && got.role == UserRole::CommitteeOperator
        ));
    }

    #[tokio::test]
    async fn test_admin_email() {
        let user_id = "test_user_id".to_string();
        let email = test::model::ADMINISTRATOR_EMAIL.clone().into_string();

        let app = test::build_mock_app()
            .build()
            .authenticate_as(user_id.clone(), email);

        let input = mock_input();
        assert!(matches!(
            signup::run(&app, input).await,
            Ok(got)
            if got.id == UserId(user_id)
            && got.role == UserRole::Administrator
        ));
    }
}
