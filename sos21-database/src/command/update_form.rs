use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Input {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub starts_at: DateTime<Utc>,
    pub ends_at: DateTime<Utc>,
    pub items: serde_json::Value,
}

pub async fn update_form<'a, E>(conn: E, input: Input) -> Result<()>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    sqlx::query!(
        r#"
UPDATE forms
  SET
    name = $2,
    description = $3,
    starts_at = $4,
    ends_at = $5,
    items = $6
  WHERE id = $1
"#,
        input.id,
        input.name,
        input.description,
        input.starts_at,
        input.ends_at,
        input.items,
    )
    .execute(conn)
    .await
    .context("Failed to update on forms")?;
    Ok(())
}
