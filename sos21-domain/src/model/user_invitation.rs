use crate::context::{UserInvitationRepository, UserRepository};
use crate::model::date_time::DateTime;
use crate::model::permissions::Permissions;
use crate::model::user::{self, User, UserEmailAddress, UserId};
use crate::{DomainError, DomainResult};

use anyhow::Context;
use thiserror::Error;
use uuid::Uuid;

mod role;
pub use role::UserInvitationRole;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UserInvitationId(Uuid);

impl UserInvitationId {
    pub fn from_uuid(uuid: Uuid) -> Self {
        UserInvitationId(uuid)
    }

    pub fn to_uuid(&self) -> Uuid {
        self.0
    }
}

#[derive(Debug, Clone)]
pub struct UserInvitationContent {
    pub id: UserInvitationId,
    pub created_at: DateTime,
    pub author_id: UserId,
    pub email: UserEmailAddress,
    pub role: UserInvitationRole,
}

#[derive(Debug, Clone)]
pub struct UserInvitation {
    content: UserInvitationContent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NewUserInvitationErrorKind {
    InsufficientPermissions,
    AlreadyInvitedEmailAddress,
    AlreadySignedUpEmailAddress,
}

#[derive(Debug, Clone, Error)]
#[error("failed to create a user invitation")]
pub struct NewUserInvitationError {
    kind: NewUserInvitationErrorKind,
}

impl NewUserInvitationError {
    pub fn kind(&self) -> NewUserInvitationErrorKind {
        self.kind
    }

    fn from_permissions_error(_err: user::RequirePermissionsError) -> Self {
        NewUserInvitationError {
            kind: NewUserInvitationErrorKind::InsufficientPermissions,
        }
    }
}

impl UserInvitation {
    pub async fn new<C>(
        ctx: &C,
        author: &User,
        email: UserEmailAddress,
        role: UserInvitationRole,
    ) -> DomainResult<Self, NewUserInvitationError>
    where
        C: UserInvitationRepository + UserRepository,
    {
        author
            .require_permissions(Permissions::CREATE_USER_INVITATIONS)
            .map_err(NewUserInvitationError::from_permissions_error)
            .map_err(DomainError::Domain)?;

        if ctx
            .get_user_by_email(&email)
            .await
            .context("Failed to get user")?
            .is_some()
        {
            return Err(DomainError::Domain(NewUserInvitationError {
                kind: NewUserInvitationErrorKind::AlreadySignedUpEmailAddress,
            }));
        }

        if ctx
            .get_user_invitation_by_email(&email)
            .await
            .context("Failed to get user invitation")?
            .is_some()
        {
            return Err(DomainError::Domain(NewUserInvitationError {
                kind: NewUserInvitationErrorKind::AlreadyInvitedEmailAddress,
            }));
        }

        Ok(UserInvitation::from_content(UserInvitationContent {
            id: UserInvitationId::from_uuid(Uuid::new_v4()),
            created_at: DateTime::now(),
            author_id: author.id().clone(),
            email,
            role,
        }))
    }

    /// Restore `UserInvitation` from `UserInvitationContent`.
    ///
    /// This is intended to be used when the data is taken out of the implementation
    /// by [`UserInvitation::into_content`] for persistence, internal serialization, etc.
    /// Use [`UserInvitation::new`] to create an user invitation.
    pub fn from_content(content: UserInvitationContent) -> Self {
        UserInvitation { content }
    }

    /// Convert `UserInvitation` into `UserInvitationContent`.
    pub fn into_content(self) -> UserInvitationContent {
        self.content
    }

    pub fn id(&self) -> UserInvitationId {
        self.content.id
    }

    pub fn created_at(&self) -> DateTime {
        self.content.created_at
    }

    pub fn author_id(&self) -> &UserId {
        &self.content.author_id
    }

    pub fn email(&self) -> &UserEmailAddress {
        &self.content.email
    }

    pub fn role(&self) -> UserInvitationRole {
        self.content.role
    }

    pub fn is_visible_to(&self, user: &User) -> bool {
        user.permissions()
            .contains(Permissions::READ_ALL_USER_INVITATIONS)
    }
}

#[cfg(test)]
mod tests {
    use super::{NewUserInvitationErrorKind, UserInvitation, UserInvitationRole};

    use crate::model::user::UserEmailAddress;
    use crate::test::model as test_model;
    use crate::DomainError;

    #[test]
    fn test_visibility_general() {
        let admin = test_model::new_admin_user();
        let email = test_model::mock_user_email_address().into_string();

        let invitation = test_model::new_operator_user_invitation(admin.id().clone(), email);
        let general = test_model::new_general_user();
        assert!(!invitation.is_visible_to(&general));
    }

    #[test]
    fn test_visibility_operator() {
        let admin = test_model::new_admin_user();
        let email = test_model::mock_user_email_address().into_string();

        let invitation = test_model::new_operator_user_invitation(admin.id().clone(), email);
        let operator = test_model::new_operator_user();
        assert!(!invitation.is_visible_to(&operator));
    }

    #[test]
    fn test_visibility_admin() {
        let admin1 = test_model::new_admin_user();
        let email = test_model::mock_user_email_address().into_string();

        let invitation = test_model::new_operator_user_invitation(admin1.id().clone(), email);
        let admin2 = test_model::new_admin_user();
        assert!(invitation.is_visible_to(&admin2));
    }

    #[tokio::test]
    async fn test_new_general() {
        let general = test_model::new_general_user();
        let email = test_model::mock_user_email_address();

        let app = crate::test::build_mock_app()
            .users(vec![general.clone()])
            .build();
        assert!(matches!(
            UserInvitation::new(&app, &general, email, UserInvitationRole::Committee).await,
            Err(DomainError::Domain(err))
            if err.kind() == NewUserInvitationErrorKind::InsufficientPermissions
        ));
    }

    #[tokio::test]
    async fn test_new_operator() {
        let operator = test_model::new_operator_user();
        let email = test_model::mock_user_email_address();

        let app = crate::test::build_mock_app()
            .users(vec![operator.clone()])
            .build();
        assert!(matches!(
            UserInvitation::new(&app, &operator, email, UserInvitationRole::Committee).await,
            Err(DomainError::Domain(err))
            if err.kind() == NewUserInvitationErrorKind::InsufficientPermissions
        ));
    }

    #[tokio::test]
    async fn test_new_admin_already_signed_up() {
        let admin = test_model::new_admin_user();
        let email = admin.email().clone();

        let app = crate::test::build_mock_app()
            .users(vec![admin.clone()])
            .build();
        assert!(matches!(
            UserInvitation::new(&app, &admin, email, UserInvitationRole::Committee).await,
            Err(DomainError::Domain(err))
            if err.kind() == NewUserInvitationErrorKind::AlreadySignedUpEmailAddress
        ));
    }

    #[tokio::test]
    async fn test_new_admin_already_invited() {
        let admin = test_model::new_admin_user();
        let email = UserEmailAddress::from_string("example-new@s.tsukuba.ac.jp").unwrap();
        let invitation = test_model::new_operator_user_invitation(
            admin.id().clone(),
            email.clone().into_string(),
        );

        let app = crate::test::build_mock_app()
            .users(vec![admin.clone()])
            .user_invitations(vec![invitation])
            .build();
        assert!(matches!(
            UserInvitation::new(&app, &admin, email, UserInvitationRole::Committee).await,
            Err(DomainError::Domain(err))
            if err.kind() == NewUserInvitationErrorKind::AlreadyInvitedEmailAddress
        ));
    }

    #[tokio::test]
    async fn test_new_admin_ok() {
        let admin = test_model::new_admin_user();
        let email = UserEmailAddress::from_string("example-new@s.tsukuba.ac.jp").unwrap();

        let app = crate::test::build_mock_app()
            .users(vec![admin.clone()])
            .build();
        assert!(matches!(
            UserInvitation::new(&app, &admin, email.clone(), UserInvitationRole::Committee).await,
            Ok(invitation)
            if invitation.role() == UserInvitationRole::Committee
            && invitation.email() == &email
        ));
    }
}
