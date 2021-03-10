use anyhow::{Context, Result};
use uuid::Uuid;

pub async fn insert_form_condition_excludes<'a, E>(
    conn: E,
    form_id: Uuid,
    exclude_ids: Vec<Uuid>,
) -> Result<()>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    sqlx::query!(
        r#"
INSERT INTO form_condition_excludes (
    project_id,
    form_id
)
SELECT
    exclude_ids.id AS project_id,
    $1 AS form_id
FROM unnest($2::uuid[]) AS exclude_ids( id )
"#,
        form_id,
        &exclude_ids
    )
    .execute(conn)
    .await
    .context("Failed to insert to form condition excludes")?;

    Ok(())
}
