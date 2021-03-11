use anyhow::{Context, Result};
use uuid::Uuid;

pub async fn delete_form_condition_excludes<'a, E>(
    conn: E,
    form_id: Uuid,
    project_ids: Vec<Uuid>,
) -> Result<()>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    sqlx::query!(
        r#"
DELETE FROM form_condition_excludes
WHERE project_id = ANY ($2) AND form_id = $1
"#,
        form_id,
        &project_ids,
    )
    .execute(conn)
    .await
    .context("Failed to delete form condition excludes")?;

    Ok(())
}
