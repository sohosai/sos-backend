use anyhow::{Context, Result};
use uuid::Uuid;

pub async fn delete_registration_form_project_query_conjunctions<'a, E>(
    conn: E,
    registration_form_id: Uuid,
) -> Result<()>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    sqlx::query!(
        r#"
DELETE FROM registration_form_project_query_conjunctions
WHERE registration_form_id = $1
"#,
        registration_form_id,
    )
    .execute(conn)
    .await
    .context("Failed to delete from registration form project query conjunctions")?;

    Ok(())
}
