use crate::model::file::File;

use anyhow::{Context, Result};
use futures::stream::{BoxStream, StreamExt};

pub fn list_files_by_user<'a, 'b, E>(conn: E, user_id: String) -> BoxStream<'b, Result<File>>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres> + 'b,
    'a: 'b,
{
    sqlx::query_as!(File, "SELECT * FROM files WHERE author_id = $1", user_id)
        .fetch(conn)
        .map(|result| result.context("Failed to select from files"))
        .boxed()
}
