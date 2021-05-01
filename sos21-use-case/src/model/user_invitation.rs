use crate::model::user::UserId;

use chrono::{DateTime, Utc};
use sos21_domain::model::user_invitation as entity;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UserInvitationId(pub Uuid);

impl UserInvitationId {
    pub fn from_entity(id: entity::UserInvitationId) -> Self {
        UserInvitationId(id.to_uuid())
    }

    pub fn into_entity(self) -> entity::UserInvitationId {
        entity::UserInvitationId::from_uuid(self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UserInvitationRole {
    Committee,
    CommitteeOperator,
    Administrator,
}

impl UserInvitationRole {
    pub fn from_entity(role: entity::UserInvitationRole) -> Self {
        match role {
            entity::UserInvitationRole::Committee => UserInvitationRole::Committee,
            entity::UserInvitationRole::CommitteeOperator => UserInvitationRole::CommitteeOperator,
            entity::UserInvitationRole::Administrator => UserInvitationRole::Administrator,
        }
    }

    pub fn into_entity(self) -> entity::UserInvitationRole {
        match self {
            UserInvitationRole::Committee => entity::UserInvitationRole::Committee,
            UserInvitationRole::CommitteeOperator => entity::UserInvitationRole::CommitteeOperator,
            UserInvitationRole::Administrator => entity::UserInvitationRole::Administrator,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserInvitation {
    pub id: UserInvitationId,
    pub created_at: DateTime<Utc>,
    pub author_id: UserId,
    pub email: String,
    pub role: UserInvitationRole,
}

impl UserInvitation {
    pub fn from_entity(invitation: entity::UserInvitation) -> Self {
        UserInvitation {
            id: UserInvitationId::from_entity(invitation.id()),
            created_at: invitation.created_at().utc(),
            author_id: UserId::from_entity(invitation.author_id().clone()),
            email: invitation.email().clone().into_string(),
            role: UserInvitationRole::from_entity(invitation.role()),
        }
    }
}
