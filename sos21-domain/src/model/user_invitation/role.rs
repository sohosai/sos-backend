use crate::model::user::UserRole;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserInvitationRole {
    Committee,
    CommitteeOperator,
    Administrator,
}

impl UserInvitationRole {
    pub fn to_user_role(&self) -> UserRole {
        match self {
            UserInvitationRole::Committee => UserRole::Committee,
            UserInvitationRole::CommitteeOperator => UserRole::CommitteeOperator,
            UserInvitationRole::Administrator => UserRole::Administrator,
        }
    }
}
