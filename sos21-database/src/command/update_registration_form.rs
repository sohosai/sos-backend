use anyhow::{Context, Result};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Input {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub items: serde_json::Value,
}

pub async fn update_registration_form<'a, E>(conn: E, input: Input) -> Result<()>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    sqlx::query!(
        r#"
UPDATE registration_forms
  SET
    name = $2,
    description = $3,
    items = $4
  WHERE id = $1
"#,
        input.id,
        input.name,
        input.description,
        input.items,
    )
    .execute(conn)
    .await
    .context("Failed to update on registration forms")?;
    Ok(())
}
