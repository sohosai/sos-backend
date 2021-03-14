use crate::model::file::File;

use anyhow::{Context, Result};
use uuid::Uuid;

pub async fn find_file<'a, E>(conn: E, id: Uuid) -> Result<Option<File>>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    sqlx::query_as!(File, "SELECT * FROM files WHERE id = $1", id)
        .fetch_optional(conn)
        .await
        .context("Failed to select from files")
}
