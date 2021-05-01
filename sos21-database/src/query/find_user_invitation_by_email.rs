use crate::model::user_invitation::UserInvitation;

use anyhow::{Context, Result};

pub async fn find_user_invitation_by_email<'a, E, S>(
    conn: E,
    email: S,
) -> Result<Option<UserInvitation>>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    S: AsRef<str>,
{
    sqlx::query_as_unchecked!(
        UserInvitation,
        "SELECT * FROM user_invitations WHERE email = $1",
        email.as_ref(),
    )
    .fetch_optional(conn)
    .await
    .context("Failed to select from user invitations")
}
