use crate::error::{UseCaseError, UseCaseResult};
use crate::model::user::User;
use crate::model::user_invitation::{UserInvitation, UserInvitationRole};

use anyhow::Context;
use sos21_domain::context::{Login, UserInvitationRepository, UserRepository};
use sos21_domain::model::{user, user_invitation};

#[derive(Debug, Clone)]
pub enum Error {
    InvalidEmailAddress,
    NotUniversityEmailAddress,
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

    fn from_new_invitation_error(
        err: user_invitation::NewUserInvitationError,
    ) -> UseCaseError<Self> {
        match err.kind() {
            user_invitation::NewUserInvitationErrorKind::InsufficientPermissions => {
                UseCaseError::UseCase(Error::InsufficientPermissions)
            }
            user_invitation::NewUserInvitationErrorKind::AlreadyInvitedEmailAddress => {
                use_case_internal!(
                    "Unexpected NewUserInvitationErrorKind::AlreadyInvitedEmailAddress"
                )
            }
            user_invitation::NewUserInvitationErrorKind::AlreadySignedUpEmailAddress => {
                use_case_internal!(
                    "Unexpected NewUserInvitationErrorKind::AlreadySignedUpEmailAddress"
                )
            }
        }
    }

    fn from_set_role_error(_err: user::NoUpdatePermissionError) -> Self {
        Error::InsufficientPermissions
    }
}

#[derive(Debug, Clone)]
pub struct Input {
    pub email: String,
    pub role: UserInvitationRole,
}

#[derive(Debug, Clone)]
pub enum Output {
    Invitation(UserInvitation),
    User(User),
}

#[tracing::instrument(skip(ctx))]
pub async fn run<C>(ctx: &Login<C>, input: Input) -> UseCaseResult<Output, Error>
where
    C: UserInvitationRepository + UserRepository + Send + Sync,
{
    let login_user = ctx.login_user();

    let email = user::UserEmailAddress::from_string(input.email)
        .map_err(|err| UseCaseError::UseCase(Error::from_email_error(err)))?;
    let role = input.role.into_entity();

    if let Some(mut user) = ctx
        .get_user_by_email(&email)
        .await
        .context("Failed to get user")?
    {
        user.set_role(login_user, role.to_user_role())
            .map_err(|err| UseCaseError::UseCase(Error::from_set_role_error(err)))?;
        ctx.store_user(user.clone())
            .await
            .context("Failed to store user")?;

        use_case_ensure!(user.is_visible_to(login_user));
        Ok(Output::User(User::from_entity(user)))
    } else {
        if let Some(invitation) = ctx
            .get_user_invitation_by_email(&email)
            .await
            .context("Failed to get user invitation")?
        {
            if invitation.role() == role {
                use_case_ensure!(invitation.is_visible_to(login_user));
                return Ok(Output::Invitation(UserInvitation::from_entity(invitation)));
            } else {
                ctx.delete_user_invitation(invitation.id()).await?;
            }
        }

        let invitation = user_invitation::UserInvitation::new(ctx, login_user, email, role)
            .await
            .map_err(|err| {
                UseCaseError::from_domain(err, Error::from_new_invitation_error).flatten()
            })?;

        ctx.store_user_invitation(invitation.clone())
            .await
            .context("Failed to store user invitation")?;

        use_case_ensure!(invitation.is_visible_to(login_user));
        Ok(Output::Invitation(UserInvitation::from_entity(invitation)))
    }
}
