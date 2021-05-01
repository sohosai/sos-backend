use anyhow::Result;
use futures::lock::Mutex;
use futures::{future, stream::TryStreamExt};
use ref_cast::RefCast;
use sos21_database::{command, model as data, query};
use sos21_domain::context::UserInvitationRepository;
use sos21_domain::model::{
    date_time::DateTime,
    user::{UserEmailAddress, UserId},
    user_invitation::{
        UserInvitation, UserInvitationContent, UserInvitationId, UserInvitationRole,
    },
};
use sqlx::{Postgres, Transaction};

#[derive(Debug, RefCast)]
#[repr(transparent)]
pub struct UserInvitationDatabase(Mutex<Transaction<'static, Postgres>>);

#[async_trait::async_trait]
impl UserInvitationRepository for UserInvitationDatabase {
    async fn store_user_invitation(&self, invitation: UserInvitation) -> Result<()> {
        let mut lock = self.0.lock().await;

        let invitation = from_user_invitation(invitation);
        if query::find_user_invitation(&mut *lock, invitation.id)
            .await?
            .is_some()
        {
            let input = command::update_user_invitation::Input {
                id: invitation.id,
                email: invitation.email,
                role: invitation.role,
            };
            command::update_user_invitation(&mut *lock, input).await
        } else {
            command::insert_user_invitation(&mut *lock, invitation).await
        }
    }

    async fn delete_user_invitation(&self, id: UserInvitationId) -> Result<()> {
        let mut lock = self.0.lock().await;
        command::delete_user_invitation(&mut *lock, id.to_uuid()).await
    }

    async fn get_user_invitation(&self, id: UserInvitationId) -> Result<Option<UserInvitation>> {
        let mut lock = self.0.lock().await;
        query::find_user_invitation(&mut *lock, id.to_uuid())
            .await
            .and_then(|opt| opt.map(to_user_invitation).transpose())
    }

    async fn list_user_invitations(&self) -> Result<Vec<UserInvitation>> {
        let mut lock = self.0.lock().await;
        query::list_user_invitations(&mut *lock)
            .and_then(|invitation| future::ready(to_user_invitation(invitation)))
            .try_collect()
            .await
    }

    async fn get_user_invitation_by_email(
        &self,
        email: &UserEmailAddress,
    ) -> Result<Option<UserInvitation>> {
        let mut lock = self.0.lock().await;
        query::find_user_invitation_by_email(&mut *lock, email.as_str())
            .await
            .and_then(|opt| opt.map(to_user_invitation).transpose())
    }
}

fn from_user_invitation(invitation: UserInvitation) -> data::user_invitation::UserInvitation {
    let UserInvitationContent {
        id,
        created_at,
        author_id,
        email,
        role,
    } = invitation.into_content();

    data::user_invitation::UserInvitation {
        id: id.to_uuid(),
        created_at: created_at.utc(),
        author_id: author_id.0,
        email: email.into_string(),
        role: from_user_invitation_role(role),
    }
}

fn from_user_invitation_role(
    role: UserInvitationRole,
) -> data::user_invitation::UserInvitationRole {
    match role {
        UserInvitationRole::Committee => data::user_invitation::UserInvitationRole::Committee,
        UserInvitationRole::CommitteeOperator => {
            data::user_invitation::UserInvitationRole::CommitteeOperator
        }
        UserInvitationRole::Administrator => {
            data::user_invitation::UserInvitationRole::Administrator
        }
    }
}

fn to_user_invitation(invitation: data::user_invitation::UserInvitation) -> Result<UserInvitation> {
    let data::user_invitation::UserInvitation {
        id,
        created_at,
        author_id,
        email,
        role,
    } = invitation;

    Ok(UserInvitation::from_content(UserInvitationContent {
        id: UserInvitationId::from_uuid(id),
        created_at: DateTime::from_utc(created_at),
        author_id: UserId(author_id),
        email: UserEmailAddress::from_string(email)?,
        role: to_user_invitation_role(role),
    }))
}

fn to_user_invitation_role(role: data::user_invitation::UserInvitationRole) -> UserInvitationRole {
    match role {
        data::user_invitation::UserInvitationRole::Committee => UserInvitationRole::Committee,
        data::user_invitation::UserInvitationRole::CommitteeOperator => {
            UserInvitationRole::CommitteeOperator
        }
        data::user_invitation::UserInvitationRole::Administrator => {
            UserInvitationRole::Administrator
        }
    }
}
