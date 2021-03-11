use anyhow::{Context, Result};
use uuid::Uuid;

pub async fn delete_form_project_query_conjunctions<'a, E>(conn: E, form_id: Uuid) -> Result<()>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    sqlx::query!(
        r#"
DELETE FROM form_project_query_conjunctions
WHERE form_id = $1
"#,
        form_id,
    )
    .execute(conn)
    .await
    .context("Failed to delete from form project query conjunctions")?;

    Ok(())
}
