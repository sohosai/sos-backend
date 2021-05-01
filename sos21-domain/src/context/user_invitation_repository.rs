use crate::model::user::UserEmailAddress;
use crate::model::user_invitation::{UserInvitation, UserInvitationId};

use anyhow::Result;

#[async_trait::async_trait]
pub trait UserInvitationRepository {
    async fn store_user_invitation(&self, invitation: UserInvitation) -> Result<()>;
    async fn delete_user_invitation(&self, id: UserInvitationId) -> Result<()>;
    async fn get_user_invitation(&self, id: UserInvitationId) -> Result<Option<UserInvitation>>;
    // TODO: Move to query service
    async fn list_user_invitations(&self) -> Result<Vec<UserInvitation>>;
    async fn get_user_invitation_by_email(
        &self,
        email: &UserEmailAddress,
    ) -> Result<Option<UserInvitation>>;
}

#[macro_export]
macro_rules! delegate_user_invitation_repository {
    (impl $(<$($vars:ident $(: $c0:ident $(+ $cs:ident)* )? ),*>)? UserInvitationRepository for $ty:ty {
        $sel:ident $target:block
    }) => {
        #[::async_trait::async_trait]
        impl $(<$($vars$(: $c0 $(+ $cs)* )?,)*>)? $crate::context::UserInvitationRepository for $ty {
            async fn store_user_invitation(
                &$sel,
                invitation: $crate::model::user_invitation::UserInvitation,
            ) -> ::anyhow::Result<()> {
                $target.store_user_invitation(invitation).await
            }
            async fn delete_user_invitation(
                &$sel,
                id: $crate::model::user_invitation::UserInvitationId,
            ) -> ::anyhow::Result<()> {
                $target.delete_user_invitation(id).await
            }
            async fn get_user_invitation(
                &$sel,
                id: $crate::model::user_invitation::UserInvitationId,
            ) -> ::anyhow::Result<Option<$crate::model::user_invitation::UserInvitation>> {
                $target.get_user_invitation(id).await
            }
            async fn list_user_invitations(
                &$sel,
            ) -> ::anyhow::Result<Vec<$crate::model::user_invitation::UserInvitation>> {
                $target.list_user_invitations().await
            }
            async fn get_user_invitation_by_email(
                &$sel,
                email: &$crate::model::user::UserEmailAddress,
            ) -> ::anyhow::Result<Option<$crate::model::user_invitation::UserInvitation>> {
                $target.get_user_invitation_by_email(email).await
            }
        }
    };
}

#[async_trait::async_trait]
impl<C: UserInvitationRepository + Sync> UserInvitationRepository for &C {
    async fn store_user_invitation(&self, invitation: UserInvitation) -> Result<()> {
        <C as UserInvitationRepository>::store_user_invitation(self, invitation).await
    }

    async fn delete_user_invitation(&self, id: UserInvitationId) -> Result<()> {
        <C as UserInvitationRepository>::delete_user_invitation(self, id).await
    }

    async fn get_user_invitation(&self, id: UserInvitationId) -> Result<Option<UserInvitation>> {
        <C as UserInvitationRepository>::get_user_invitation(self, id).await
    }

    async fn list_user_invitations(&self) -> Result<Vec<UserInvitation>> {
        <C as UserInvitationRepository>::list_user_invitations(self).await
    }

    async fn get_user_invitation_by_email(
        &self,
        email: &UserEmailAddress,
    ) -> Result<Option<UserInvitation>> {
        <C as UserInvitationRepository>::get_user_invitation_by_email(self, email).await
    }
}
