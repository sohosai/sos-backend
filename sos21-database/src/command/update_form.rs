use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use sqlx::types::BitVec;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Input {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub starts_at: DateTime<Utc>,
    pub ends_at: DateTime<Utc>,
    pub items: Vec<u8>,
    pub condition: Vec<u8>,
    pub unspecified_query: BitVec,
    pub general_query: BitVec,
    pub stage_query: BitVec,
    pub cooking_query: BitVec,
    pub food_query: BitVec,
    pub needs_sync: bool,
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
    items = $6,
    condition = $7,
    unspecified_query = $8,
    general_query = $9,
    stage_query = $10,
    cooking_query = $11,
    food_query = $12,
    needs_sync = $13
  WHERE id = $1
"#,
        input.id,
        input.name,
        input.description,
        input.starts_at,
        input.ends_at,
        input.items,
        input.condition,
        input.unspecified_query,
        input.general_query,
        input.stage_query,
        input.cooking_query,
        input.food_query,
        input.needs_sync,
    )
    .execute(conn)
    .await
    .context("Failed to update on forms")?;
    Ok(())
}
