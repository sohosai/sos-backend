use anyhow::{Context, Result};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Input {
    pub id: Uuid,
    pub project_id: Option<Uuid>,
    pub pending_project_id: Option<Uuid>,
    pub items: serde_json::Value,
}

pub async fn update_registration_form_answer<'a, E>(conn: E, input: Input) -> Result<()>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    sqlx::query!(
        r#"
UPDATE registration_form_answers
  SET
    project_id = $2,
    pending_project_id = $3,
    items = $4
  WHERE id = $1
"#,
        input.id,
        input.project_id,
        input.pending_project_id,
        input.items
    )
    .execute(conn)
    .await
    .context("Failed to update on registration form answers")?;

    Ok(())
}
