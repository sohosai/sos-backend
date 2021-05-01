use crate::error::{UseCaseError, UseCaseResult};
use crate::model::user_invitation::{UserInvitation, UserInvitationId};

use anyhow::Context;
use sos21_domain::context::{Login, UserInvitationRepository};
use sos21_domain::model::{permissions, user};

#[derive(Debug, Clone)]
pub enum Error {
    NotFound,
    InsufficientPermissions,
}

impl Error {
    fn from_permissions_error(_err: user::RequirePermissionsError) -> Self {
        Error::InsufficientPermissions
    }
}

#[tracing::instrument(skip(ctx))]
pub async fn run<C>(ctx: &Login<C>, id: UserInvitationId) -> UseCaseResult<UserInvitation, Error>
where
    C: UserInvitationRepository + Send + Sync,
{
    let login_user = ctx.login_user();

    login_user
        .require_permissions(permissions::Permissions::READ_ALL_USER_INVITATIONS)
        .map_err(|err| UseCaseError::UseCase(Error::from_permissions_error(err)))?;

    let result = ctx
        .get_user_invitation(id.into_entity())
        .await
        .context("Failed to get user invitation")?;
    let invitation = match result {
        Some(invitation) => invitation,
        None => return Err(UseCaseError::UseCase(Error::NotFound)),
    };

    use_case_ensure!(invitation.is_visible_to(login_user));
    Ok(UserInvitation::from_entity(invitation))
}

#[cfg(test)]
mod tests {
    use crate::model::user_invitation::UserInvitationId;
    use crate::{get_user_invitation, UseCaseError};

    use sos21_domain::test;

    #[tokio::test]
    async fn test_general() {
        let admin = test::model::new_admin_user();
        let email = test::model::mock_user_email_address().into_string();
        let invitation = test::model::new_operator_user_invitation(admin.id().clone(), email);
        let general = test::model::new_general_user();

        let app = test::build_mock_app()
            .users(vec![admin, general.clone()])
            .user_invitations(vec![invitation.clone()])
            .build()
            .login_as(general)
            .await;

        assert!(matches!(
            get_user_invitation::run(&app, UserInvitationId::from_entity(invitation.id())).await,
            Err(UseCaseError::UseCase(
                get_user_invitation::Error::InsufficientPermissions
            ))
        ));
    }

    #[tokio::test]
    async fn test_operator() {
        let admin = test::model::new_admin_user();
        let email = test::model::mock_user_email_address().into_string();
        let invitation = test::model::new_operator_user_invitation(admin.id().clone(), email);
        let operator = test::model::new_operator_user();

        let app = test::build_mock_app()
            .users(vec![admin, operator.clone()])
            .user_invitations(vec![invitation.clone()])
            .build()
            .login_as(operator)
            .await;

        assert!(matches!(
            get_user_invitation::run(&app, UserInvitationId::from_entity(invitation.id())).await,
            Err(UseCaseError::UseCase(
                get_user_invitation::Error::InsufficientPermissions
            ))
        ));
    }

    #[tokio::test]
    async fn test_admin() {
        let admin1 = test::model::new_admin_user();
        let email = test::model::mock_user_email_address().into_string();
        let invitation = test::model::new_operator_user_invitation(admin1.id().clone(), email);
        let admin2 = test::model::new_admin_user();

        let app = test::build_mock_app()
            .users(vec![admin1, admin2.clone()])
            .user_invitations(vec![invitation.clone()])
            .build()
            .login_as(admin2)
            .await;

        let invitation_id = UserInvitationId::from_entity(invitation.id());
        assert!(matches!(
            get_user_invitation::run(&app, invitation_id).await,
            Ok(got)
            if got.id == invitation_id
        ));
    }
}
