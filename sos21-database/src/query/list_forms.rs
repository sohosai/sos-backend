use crate::model::form::Form;

use anyhow::Result;
use futures::stream::{BoxStream, StreamExt, TryStreamExt};

pub fn list_forms<'a, E>(conn: E) -> BoxStream<'a, Result<Form>>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres> + 'a,
{
    sqlx::query_as!(Form, "SELECT * FROM forms")
        .fetch(conn)
        .map_err(anyhow::Error::msg)
        .boxed()
}
