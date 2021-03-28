use anyhow::{Context, Result};
use uuid::Uuid;

pub async fn delete_file_distribution_files<'a, E>(conn: E, distribution_id: Uuid) -> Result<()>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    sqlx::query!(
        r#"
DELETE FROM file_distribution_files
WHERE distribution_id = $1
"#,
        distribution_id,
    )
    .execute(conn)
    .await
    .context("Failed to delete from file distribution files")?;

    Ok(())
}
