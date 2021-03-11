use anyhow::{Context, Result};
use uuid::Uuid;

pub async fn insert_form_condition_includes<'a, E>(
    conn: E,
    form_id: Uuid,
    include_ids: Vec<Uuid>,
) -> Result<()>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    sqlx::query!(
        r#"
INSERT INTO form_condition_includes (
    project_id,
    form_id
)
SELECT
    include_ids.id AS project_id,
    $1 AS form_id
FROM unnest($2::uuid[]) AS include_ids( id )
"#,
        form_id,
        &include_ids
    )
    .execute(conn)
    .await
    .context("Failed to insert to form condition includes")?;

    Ok(())
}
