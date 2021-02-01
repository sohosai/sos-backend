use crate::error::{UseCaseError, UseCaseResult};
use crate::model::user::User;

use anyhow::Context;
use sos21_domain::context::{Login, UserRepository};
use sos21_domain::model::permissions::Permissions;

#[derive(Debug, Clone)]
pub enum Error {
    InsufficientPermissions,
}

#[tracing::instrument(skip(ctx))]
pub async fn run<C>(ctx: &Login<C>) -> UseCaseResult<Vec<User>, Error>
where
    Login<C>: UserRepository,
{
    let login_user = ctx.login_user();

    if login_user
        .require_permissions(Permissions::READ_ALL_USERS)
        .is_err()
    {
        return Err(UseCaseError::UseCase(Error::InsufficientPermissions));
    }

    let users = ctx.list_users().await.context("Failed to list users")?;
    use_case_ensure!(users.iter().all(|user| user.is_visible_to(login_user)));
    Ok(users.into_iter().map(User::from_entity).collect())
}

#[cfg(test)]
mod tests {
    use crate::model::user::UserId;
    use crate::{list_users, UseCaseError};
    use sos21_domain_test as test;

    // Checks that the normal user cannot list users.
    #[tokio::test]
    async fn test_general() {
        let user = test::model::new_general_user();
        let other = test::model::new_general_user();

        let app = test::build_mock_app()
            .users(vec![user.clone(), other.clone()])
            .build()
            .login_as(user.clone())
            .await;

        assert!(matches!(
            list_users::run(&app).await,
            Err(UseCaseError::UseCase(
                list_users::Error::InsufficientPermissions
            ))
        ));
    }

    // Checks that the (unprivileged) committee user cannot list users.
    #[tokio::test]
    async fn test_committee() {
        let user = test::model::new_committee_user();
        let other = test::model::new_general_user();

        let app = test::build_mock_app()
            .users(vec![user.clone(), other.clone()])
            .build()
            .login_as(user.clone())
            .await;

        assert!(matches!(
            list_users::run(&app).await,
            Err(UseCaseError::UseCase(
                list_users::Error::InsufficientPermissions
            ))
        ));
    }

    // Checks that the privileged committee user can list users.
    #[tokio::test]
    async fn test_operator() {
        use std::collections::HashSet;

        let user = test::model::new_operator_user();
        let other = test::model::new_general_user();

        let app = test::build_mock_app()
            .users(vec![user.clone(), other.clone()])
            .build()
            .login_as(user.clone())
            .await;

        let result = list_users::run(&app).await;
        assert!(result.is_ok());

        let got: HashSet<_> = result.unwrap().into_iter().map(|user| user.id).collect();
        let expected: HashSet<_> =
            vec![UserId::from_entity(user.id), UserId::from_entity(other.id)]
                .into_iter()
                .collect();
        assert_eq!(got, expected);
    }
}
