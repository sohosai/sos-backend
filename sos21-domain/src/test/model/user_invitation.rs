use crate::model::{
    date_time::DateTime,
    user::{UserEmailAddress, UserId},
    user_invitation::{
        UserInvitation, UserInvitationContent, UserInvitationId, UserInvitationRole,
    },
};

use uuid::Uuid;

pub fn new_user_invitation_id() -> UserInvitationId {
    UserInvitationId::from_uuid(Uuid::new_v4())
}

pub fn new_user_invitation<S>(
    author_id: UserId,
    email: S,
    role: UserInvitationRole,
) -> UserInvitation
where
    S: Into<String>,
{
    UserInvitation::from_content(UserInvitationContent {
        id: new_user_invitation_id(),
        created_at: DateTime::now(),
        author_id,
        email: UserEmailAddress::from_string(email).unwrap(),
        role,
    })
}

pub fn new_committee_user_invitation<S>(author_id: UserId, email: S) -> UserInvitation
where
    S: Into<String>,
{
    new_user_invitation(author_id, email, UserInvitationRole::Committee)
}

pub fn new_operator_user_invitation<S>(author_id: UserId, email: S) -> UserInvitation
where
    S: Into<String>,
{
    new_user_invitation(author_id, email, UserInvitationRole::CommitteeOperator)
}

pub fn new_admin_user_invitation<S>(author_id: UserId, email: S) -> UserInvitation
where
    S: Into<String>,
{
    new_user_invitation(author_id, email, UserInvitationRole::Administrator)
}
