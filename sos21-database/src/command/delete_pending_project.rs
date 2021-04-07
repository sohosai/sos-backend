use anyhow::{Context, Result};
use uuid::Uuid;

pub async fn delete_pending_project<'a, E>(conn: E, id: Uuid) -> Result<()>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    sqlx::query!(
        r#"
DELETE FROM pending_projects
WHERE id = $1
"#,
        id,
    )
    .execute(conn)
    .await
    .context("Failed to delete from pending projects")?;

    Ok(())
}
