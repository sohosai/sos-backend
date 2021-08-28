use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Input {
    pub id: Uuid,
    pub updated_at: DateTime<Utc>,
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
    updated_at = $2,
    project_id = $3,
    pending_project_id = $4,
    items = $5
  WHERE id = $1
"#,
        input.id,
        input.updated_at,
        input.project_id,
        input.pending_project_id,
        input.items
    )
    .execute(conn)
    .await
    .context("Failed to update on registration form answers")?;

    Ok(())
}
