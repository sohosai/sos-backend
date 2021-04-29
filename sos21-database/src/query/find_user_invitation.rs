use crate::model::user_invitation::UserInvitation;

use anyhow::{Context, Result};
use uuid::Uuid;

pub async fn find_user_invitation<'a, E>(conn: E, id: Uuid) -> Result<Option<UserInvitation>>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    sqlx::query_as_unchecked!(
        UserInvitation,
        "SELECT * FROM user_invitations WHERE id = $1",
        id
    )
    .fetch_optional(conn)
    .await
    .context("Failed to select from user invitations")
}
