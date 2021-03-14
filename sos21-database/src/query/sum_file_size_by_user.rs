use anyhow::{Context, Result};

pub async fn sum_file_size_by_user<'a, E>(conn: E, author_id: String) -> Result<i64>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    sqlx::query_scalar!(
        "SELECT sum(size)::bigint FROM files WHERE author_id = $1",
        author_id
    )
    .fetch_one(conn)
    .await
    .context("Failed to select from files")
    .map(|opt| opt.unwrap_or(0))
}
