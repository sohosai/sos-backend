use crate::error::{UseCaseError, UseCaseResult};
use crate::model::user_invitation::UserInvitation;

use anyhow::Context;
use sos21_domain::context::{Login, UserInvitationRepository};
use sos21_domain::model::{permissions::Permissions, user};

#[derive(Debug, Clone)]
pub enum Error {
    InsufficientPermissions,
}

impl Error {
    fn from_permissions_error(_err: user::RequirePermissionsError) -> Self {
        Error::InsufficientPermissions
    }
}

#[tracing::instrument(skip(ctx))]
pub async fn run<C>(ctx: &Login<C>) -> UseCaseResult<Vec<UserInvitation>, Error>
where
    C: UserInvitationRepository + Send + Sync,
{
    let login_user = ctx.login_user();

    login_user
        .require_permissions(Permissions::READ_ALL_USER_INVITATIONS)
        .map_err(|err| UseCaseError::UseCase(Error::from_permissions_error(err)))?;

    ctx.list_user_invitations()
        .await
        .context("Failed to list user invitations")?
        .into_iter()
        .map(|invitation| {
            use_case_ensure!(invitation.is_visible_to(login_user));
            Ok(UserInvitation::from_entity(invitation))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::model::user_invitation::UserInvitationId;
    use crate::{list_all_user_invitations, UseCaseError};

    use sos21_domain::test;

    #[tokio::test]
    async fn test_general() {
        let user = test::model::new_general_user();

        let app = test::build_mock_app()
            .users(vec![user.clone()])
            .build()
            .login_as(user)
            .await;

        assert!(matches!(
            list_all_user_invitations::run(&app).await,
            Err(UseCaseError::UseCase(
                list_all_user_invitations::Error::InsufficientPermissions
            ))
        ));
    }

    #[tokio::test]
    async fn test_admin_list() {
        use std::collections::HashSet;

        let admin = test::model::new_admin_user();
        let invitation1 = test::model::new_operator_user_invitation(
            admin.id().clone(),
            "example1@s.tsukuba.ac.jp",
        );
        let invitation2 = test::model::new_operator_user_invitation(
            admin.id().clone(),
            "example2@s.tsukuba.ac.jp",
        );

        let app = test::build_mock_app()
            .users(vec![admin.clone()])
            .user_invitations(vec![invitation1.clone(), invitation2.clone()])
            .build()
            .login_as(admin)
            .await;

        let invitations = list_all_user_invitations::run(&app).await.unwrap();
        let got: HashSet<_> = invitations
            .into_iter()
            .map(|invitation| invitation.id)
            .collect();
        let expected: HashSet<_> = vec![invitation1.id(), invitation2.id()]
            .into_iter()
            .map(UserInvitationId::from_entity)
            .collect();
        assert_eq!(got, expected);
    }
}
