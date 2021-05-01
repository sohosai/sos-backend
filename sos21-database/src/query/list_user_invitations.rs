use crate::model::user_invitation::UserInvitation;

use anyhow::{Context, Result};
use futures::stream::{BoxStream, StreamExt};

pub fn list_user_invitations<'a, E>(conn: E) -> BoxStream<'a, Result<UserInvitation>>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres> + 'a,
{
    sqlx::query_as_unchecked!(UserInvitation, "SELECT * FROM user_invitations")
        .fetch(conn)
        .map(|result| result.context("Failed to select from user invitations"))
        .boxed()
}
