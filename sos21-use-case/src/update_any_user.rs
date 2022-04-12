use crate::error::{UseCaseError, UseCaseResult};
use crate::model::user::{User, UserCategory, UserId, UserKanaName, UserName, UserRole};

use anyhow::Context;
use sos21_domain::context::{Login, UserRepository};
use sos21_domain::model::{permissions::Permissions, phone_number, user};

#[derive(Debug, Clone)]
pub enum Error {
    NotFound,
    InvalidName,
    InvalidKanaName,
    InvalidPhoneNumber,
    InsufficientPermissions,
}

impl Error {
    fn from_name_error(_err: user::name::NameError) -> Self {
        Error::InvalidName
    }

    fn from_kana_name_error(_err: user::name::KanaNameError) -> Self {
        Error::InvalidKanaName
    }

    fn from_phone_number_error(_err: phone_number::FromStringError) -> Self {
        Error::InvalidPhoneNumber
    }

    fn from_permissions_error(_err: user::RequirePermissionsError) -> Self {
        Error::InsufficientPermissions
    }

    fn from_update_error(_err: user::NoUpdatePermissionError) -> Self {
        Error::InsufficientPermissions
    }
}

#[derive(Debug, Clone)]
pub struct Input {
    pub id: UserId,
    pub name: Option<UserName>,
    pub kana_name: Option<UserKanaName>,
    pub phone_number: Option<String>,
    pub role: Option<UserRole>,
    pub category: Option<UserCategory>,
}

#[tracing::instrument(skip(ctx))]
pub async fn run<C>(ctx: &Login<C>, input: Input) -> UseCaseResult<User, Error>
where
    C: UserRepository + Send + Sync,
{
    let login_user = ctx.login_user();

    login_user
        .require_permissions(Permissions::UPDATE_ALL_USERS)
        .map_err(|err| UseCaseError::UseCase(Error::from_permissions_error(err)))?;

    let user = ctx
        .get_user(input.id.clone().into_entity())
        .await
        .context("Failed to get a user")?;
    let mut user = match user {
        Some(x) => x,
        None => return Err(UseCaseError::UseCase(Error::NotFound)),
    };

    if let Some(name) = input.name {
        let name = user::UserName::from_string(name.first, name.last)
            .map_err(|err| UseCaseError::UseCase(Error::from_name_error(err)))?;
        user.set_name(login_user, name)
            .map_err(|err| UseCaseError::UseCase(Error::from_update_error(err)))?;
    }

    if let Some(kana_name) = input.kana_name {
        let kana_name = user::UserKanaName::from_string(kana_name.first, kana_name.last)
            .map_err(|err| UseCaseError::UseCase(Error::from_kana_name_error(err)))?;
        user.set_kana_name(login_user, kana_name)
            .map_err(|err| UseCaseError::UseCase(Error::from_update_error(err)))?;
    }

    if let Some(phone_number) = input.phone_number {
        let phone_number = phone_number::PhoneNumber::from_string(phone_number)
            .map_err(|err| UseCaseError::UseCase(Error::from_phone_number_error(err)))?;
        user.set_phone_number(login_user, phone_number)
            .map_err(|err| UseCaseError::UseCase(Error::from_update_error(err)))?;
    }

    if let Some(role) = input.role {
        user.set_role(login_user, role.into_entity())
            .map_err(|err| UseCaseError::UseCase(Error::from_update_error(err)))?;
    }

    if let Some(category) = input.category {
        let category = match category {
            UserCategory::UndergraduateStudent=>user::UserCategory::UndergraduateStudent,
            UserCategory::GraduateStudent => user::UserCategory::GraduateStudent,
            UserCategory::AcademicStaff => user::UserCategory::AcademicStaff,
        };
        user.set_category(login_user, category)
            .map_err(|err| UseCaseError::UseCase(Error::from_update_error(err)))?;
    }

    ctx.store_user(user.clone())
        .await
        .context("Failed to store a updated user")?;
    use_case_ensure!(user.is_visible_to(login_user));
    Ok(User::from_entity(user))
}

#[cfg(test)]
mod tests {
    use crate::model::user::{UserId, UserRole};
    use crate::{update_any_user, UseCaseError};
    use sos21_domain::test;

    // Checks that the normal user cannot update users.
    #[tokio::test]
    async fn test_general() {
        let user = test::model::new_general_user();

        let app = test::build_mock_app()
            .users(vec![user.clone()])
            .build()
            .login_as(user.clone())
            .await;

        let input = update_any_user::Input {
            id: UserId::from_entity(user.id().clone()),
            name: None,
            kana_name: None,
            phone_number: None,
            role: Some(UserRole::Administrator),
            category: None,
        };
        assert!(matches!(
            update_any_user::run(&app, input).await,
            Err(UseCaseError::UseCase(
                update_any_user::Error::InsufficientPermissions
            ))
        ));
    }

    // Checks that the (unprivileged) committee user cannot update users.
    #[tokio::test]
    async fn test_committee() {
        let user = test::model::new_committee_user();

        let app = test::build_mock_app()
            .users(vec![user.clone()])
            .build()
            .login_as(user.clone())
            .await;

        let input = update_any_user::Input {
            id: UserId::from_entity(user.id().clone()),
            name: None,
            kana_name: None,
            phone_number: None,
            role: Some(UserRole::General),
            category: None,
        };
        assert!(matches!(
            update_any_user::run(&app, input).await,
            Err(UseCaseError::UseCase(
                update_any_user::Error::InsufficientPermissions
            ))
        ));
    }

    // Checks that the privileged committee user cannot update users.
    #[tokio::test]
    async fn test_operator() {
        let user = test::model::new_operator_user();

        let app = test::build_mock_app()
            .users(vec![user.clone()])
            .build()
            .login_as(user.clone())
            .await;

        let input = update_any_user::Input {
            id: UserId::from_entity(user.id().clone()),
            name: None,
            kana_name: None,
            phone_number: None,
            role: Some(UserRole::Committee),
            category: None,
        };
        assert!(matches!(
            update_any_user::run(&app, input).await,
            Err(UseCaseError::UseCase(
                update_any_user::Error::InsufficientPermissions
            ))
        ));
    }

    // Checks that the administrator can update users.
    #[tokio::test]
    async fn test_admin() {
        let user = test::model::new_admin_user();

        let app = test::build_mock_app()
            .users(vec![user.clone()])
            .build()
            .login_as(user.clone())
            .await;

        let input = update_any_user::Input {
            id: UserId::from_entity(user.id().clone()),
            name: None,
            kana_name: None,
            phone_number: None,
            role: Some(UserRole::CommitteeOperator),
            category: None,
        };
        assert!(matches!(
            update_any_user::run(&app, input).await,
            Ok(got)
            if got.role == UserRole::CommitteeOperator
        ));
    }
}
