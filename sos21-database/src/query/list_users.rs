use crate::model::user::User;

use anyhow::Result;
use futures::stream::{BoxStream, StreamExt, TryStreamExt};

pub fn list_users<'a, E>(conn: E) -> BoxStream<'a, Result<User>>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres> + 'a,
{
    sqlx::query_as("SELECT * FROM users")
        .fetch(conn)
        .map_err(anyhow::Error::msg)
        .boxed()
}
