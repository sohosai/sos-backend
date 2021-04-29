use crate::error::{UseCaseError, UseCaseResult};
use crate::model::user_invitation::{UserInvitation, UserInvitationRole};

use anyhow::Context;
use sos21_domain::context::{Login, UserInvitationRepository, UserRepository};
use sos21_domain::model::{user, user_invitation};

#[derive(Debug, Clone)]
pub enum Error {
    InvalidEmailAddress,
    NotUniversityEmailAddress,
    AlreadyInvitedEmailAddress,
    AlreadySignedUpEmailAddress,
    InsufficientPermissions,
}

impl Error {
    fn from_email_error(err: user::email::EmailAddressError) -> Self {
        match err.kind() {
            user::email::EmailAddressErrorKind::NotUniversityEmailAddress => {
                Error::NotUniversityEmailAddress
            }
            user::email::EmailAddressErrorKind::InvalidEmailAddress => Error::InvalidEmailAddress,
        }
    }

    fn from_new_invitation_error(err: user_invitation::NewUserInvitationError) -> Self {
        match err.kind() {
            user_invitation::NewUserInvitationErrorKind::InsufficientPermissions => {
                Error::InsufficientPermissions
            }
            user_invitation::NewUserInvitationErrorKind::AlreadyInvitedEmailAddress => {
                Error::AlreadyInvitedEmailAddress
            }
            user_invitation::NewUserInvitationErrorKind::AlreadySignedUpEmailAddress => {
                Error::AlreadySignedUpEmailAddress
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Input {
    pub email: String,
    pub role: UserInvitationRole,
}

// TODO: Actually send an invitation email
#[tracing::instrument(skip(ctx))]
pub async fn run<C>(ctx: &Login<C>, input: Input) -> UseCaseResult<UserInvitation, Error>
where
    C: UserInvitationRepository + UserRepository + Send + Sync,
{
    let login_user = ctx.login_user();

    let email = user::UserEmailAddress::from_string(input.email)
        .map_err(|err| UseCaseError::UseCase(Error::from_email_error(err)))?;

    let invitation =
        user_invitation::UserInvitation::new(ctx, login_user, email, input.role.into_entity())
            .await
            .map_err(|err| UseCaseError::from_domain(err, Error::from_new_invitation_error))?;

    ctx.store_user_invitation(invitation.clone())
        .await
        .context("Failed to store user invitation")?;

    use_case_ensure!(invitation.is_visible_to(login_user));
    Ok(UserInvitation::from_entity(invitation))
}

#[cfg(test)]
mod tests {
    use crate::model::user_invitation::UserInvitationRole;
    use crate::{invite_user, UseCaseError};

    use sos21_domain::test;

    #[tokio::test]
    async fn test_operator() {
        let user = test::model::new_operator_user();

        let app = test::build_mock_app()
            .users(vec![user.clone()])
            .build()
            .login_as(user)
            .await;

        let input = invite_user::Input {
            email: "example-invite-user@s.tsukuba.ac.jp".to_string(),
            role: UserInvitationRole::CommitteeOperator,
        };
        assert!(matches!(
            invite_user::run(&app, input).await,
            Err(UseCaseError::UseCase(
                invite_user::Error::InsufficientPermissions
            ))
        ));
    }

    #[tokio::test]
    async fn test_already_invited() {
        let user = test::model::new_admin_user();
        let invitation = test::model::new_operator_user_invitation(
            user.id().clone(),
            "example1@s.tsukuba.ac.jp",
        );

        let app = test::build_mock_app()
            .users(vec![user.clone()])
            .user_invitations(vec![invitation.clone()])
            .build()
            .login_as(user)
            .await;

        let input = invite_user::Input {
            email: invitation.email().clone().into_string(),
            role: UserInvitationRole::CommitteeOperator,
        };
        assert!(matches!(
            invite_user::run(&app, input).await,
            Err(UseCaseError::UseCase(
                invite_user::Error::AlreadyInvitedEmailAddress
            ))
        ));
    }

    #[tokio::test]
    async fn test_already_signed_up() {
        let user = test::model::new_admin_user();

        let app = test::build_mock_app()
            .users(vec![user.clone()])
            .build()
            .login_as(user.clone())
            .await;

        let input = invite_user::Input {
            email: user.email().clone().into_string(),
            role: UserInvitationRole::CommitteeOperator,
        };
        assert!(matches!(
            invite_user::run(&app, input).await,
            Err(UseCaseError::UseCase(
                invite_user::Error::AlreadySignedUpEmailAddress
            ))
        ));
    }

    #[tokio::test]
    async fn test_admin() {
        let user = test::model::new_admin_user();

        let app = test::build_mock_app()
            .users(vec![user.clone()])
            .build()
            .login_as(user)
            .await;

        let email = "example-invite-user@s.tsukuba.ac.jp".to_string();
        let input = invite_user::Input {
            email: email.clone(),
            role: UserInvitationRole::CommitteeOperator,
        };
        assert!(matches!(
            invite_user::run(&app, input).await,
            Ok(invitation)
            if invitation.email == email
        ));
    }
}
