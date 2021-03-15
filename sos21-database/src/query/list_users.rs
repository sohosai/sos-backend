use crate::model::user::User;

use anyhow::{Context, Result};
use futures::stream::{BoxStream, StreamExt};

pub fn list_users<'a, E>(conn: E) -> BoxStream<'a, Result<User>>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres> + 'a,
{
    sqlx::query_as_unchecked!(User, "SELECT * FROM users")
        .fetch(conn)
        .map(|result| result.context("Failed to select from users"))
        .boxed()
}
