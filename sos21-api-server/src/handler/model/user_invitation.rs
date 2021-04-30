use crate::handler::model::date_time::DateTime;
use crate::handler::model::user::UserId;

use serde::{Deserialize, Serialize};
use sos21_use_case::model::user_invitation as use_case;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct UserInvitationId(pub Uuid);

impl UserInvitationId {
    pub fn from_use_case(id: use_case::UserInvitationId) -> Self {
        UserInvitationId(id.0)
    }

    pub fn into_use_case(self) -> use_case::UserInvitationId {
        use_case::UserInvitationId(self.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UserInvitationRole {
    Administrator,
    CommitteeOperator,
    Committee,
}

impl UserInvitationRole {
    pub fn from_use_case(role: use_case::UserInvitationRole) -> Self {
        match role {
            use_case::UserInvitationRole::Administrator => UserInvitationRole::Administrator,
            use_case::UserInvitationRole::CommitteeOperator => {
                UserInvitationRole::CommitteeOperator
            }
            use_case::UserInvitationRole::Committee => UserInvitationRole::Committee,
        }
    }

    pub fn into_use_case(self) -> use_case::UserInvitationRole {
        match self {
            UserInvitationRole::Administrator => use_case::UserInvitationRole::Administrator,
            UserInvitationRole::CommitteeOperator => {
                use_case::UserInvitationRole::CommitteeOperator
            }
            UserInvitationRole::Committee => use_case::UserInvitationRole::Committee,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInvitation {
    pub id: UserInvitationId,
    pub created_at: DateTime,
    pub author_id: UserId,
    pub email: String,
    pub role: UserInvitationRole,
}

impl UserInvitation {
    pub fn from_use_case(user: use_case::UserInvitation) -> Self {
        UserInvitation {
            id: UserInvitationId::from_use_case(user.id),
            created_at: DateTime::from_use_case(user.created_at),
            author_id: UserId::from_use_case(user.author_id),
            email: user.email,
            role: UserInvitationRole::from_use_case(user.role),
        }
    }
}
