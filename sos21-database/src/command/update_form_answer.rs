use anyhow::{Context, Result};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Input {
    pub id: Uuid,
    pub items: Vec<u8>,
}

pub async fn update_form_answer<'a, E>(conn: E, input: Input) -> Result<()>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    sqlx::query!(
        r#"
UPDATE form_answers
  SET
    items = $2
  WHERE id = $1
"#,
        input.id,
        input.items
    )
    .execute(conn)
    .await
    .context("Failed to update on form answers")?;
    Ok(())
}
