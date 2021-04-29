use crate::error::{UseCaseError, UseCaseResult};
use crate::model::user::{User, UserCategory, UserId, UserKanaName, UserName, UserRole};

use anyhow::Context;
use sos21_domain::context::{Login, UserRepository};
use sos21_domain::model::{permissions::Permissions, phone_number, user};

#[derive(Debug, Clone)]
pub enum Error {
    NotFound,
    InvalidUserName,
    InvalidUserKanaName,
    InvalidPhoneNumber,
    InvalidUserAffiliation,
    InsufficientPermissions,
}

#[derive(Debug, Clone)]
pub struct Input {
    pub id: UserId,
    pub name: Option<UserName>,
    pub kana_name: Option<UserKanaName>,
    pub phone_number: Option<String>,
    pub affiliation: Option<String>,
    pub role: Option<UserRole>,
    pub category: Option<UserCategory>,
}

#[tracing::instrument(skip(ctx))]
pub async fn run<C>(ctx: &Login<C>, input: Input) -> UseCaseResult<User, Error>
where
    Login<C>: UserRepository,
{
    let login_user = ctx.login_user();

    if login_user
        .require_permissions(Permissions::UPDATE_ALL_USERS)
        .is_err()
    {
        return Err(UseCaseError::UseCase(Error::InsufficientPermissions));
    }

    let user = ctx
        .get_user(input.id.clone().into_entity())
        .await
        .context("Failed to get a user")?;
    let mut user = match user {
        Some(x) => x,
        None => return Err(UseCaseError::UseCase(Error::NotFound)),
    };

    if let Some(name) = input.name {
        let name = name
            .into_entity()
            .ok_or(UseCaseError::UseCase(Error::InvalidUserName))?;
        user.set_name(name);
    }

    if let Some(kana_name) = input.kana_name {
        let kana_name = kana_name
            .into_entity()
            .ok_or(UseCaseError::UseCase(Error::InvalidUserKanaName))?;
        user.set_kana_name(kana_name);
    }

    if let Some(phone_number) = input.phone_number {
        let phone_number = phone_number::PhoneNumber::from_string(phone_number)
            .map_err(|_| UseCaseError::UseCase(Error::InvalidPhoneNumber))?;
        user.set_phone_number(phone_number);
    }

    if let Some(affiliation) = input.affiliation {
        let affiliation = user::UserAffiliation::from_string(affiliation)
            .map_err(|_| UseCaseError::UseCase(Error::InvalidUserAffiliation))?;
        user.set_affiliation(affiliation);
    }

    if let Some(role) = input.role {
        user.set_role(role.into_entity());
    }

    if let Some(category) = input.category {
        user.set_category(category.into_entity());
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
    use crate::{update_user, UseCaseError};
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

        let input = update_user::Input {
            id: UserId::from_entity(user.id().clone()),
            name: None,
            kana_name: None,
            phone_number: None,
            affiliation: None,
            role: Some(UserRole::Administrator),
            category: None,
        };
        assert!(matches!(
            update_user::run(&app, input).await,
            Err(UseCaseError::UseCase(
                update_user::Error::InsufficientPermissions
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

        let input = update_user::Input {
            id: UserId::from_entity(user.id().clone()),
            name: None,
            kana_name: None,
            phone_number: None,
            affiliation: None,
            role: Some(UserRole::General),
            category: None,
        };
        assert!(matches!(
            update_user::run(&app, input).await,
            Err(UseCaseError::UseCase(
                update_user::Error::InsufficientPermissions
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

        let input = update_user::Input {
            id: UserId::from_entity(user.id().clone()),
            name: None,
            kana_name: None,
            phone_number: None,
            affiliation: None,
            role: Some(UserRole::Committee),
            category: None,
        };
        assert!(matches!(
            update_user::run(&app, input).await,
            Err(UseCaseError::UseCase(
                update_user::Error::InsufficientPermissions
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

        let input = update_user::Input {
            id: UserId::from_entity(user.id().clone()),
            name: None,
            kana_name: None,
            phone_number: None,
            affiliation: None,
            role: Some(UserRole::CommitteeOperator),
            category: None,
        };
        assert!(matches!(
            update_user::run(&app, input).await,
            Ok(got)
            if got.role == UserRole::CommitteeOperator
        ));
    }
}
