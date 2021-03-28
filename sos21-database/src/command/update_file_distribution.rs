use anyhow::{Context, Result};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Input {
    pub id: Uuid,
    pub name: String,
    pub description: String,
}

pub async fn update_file_distribution<'a, E>(conn: E, input: Input) -> Result<()>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    sqlx::query!(
        r#"
UPDATE file_distributions
  SET
    name = $2,
    description = $3
  WHERE id = $1
"#,
        input.id,
        input.name,
        input.description
    )
    .execute(conn)
    .await
    .context("Failed to update on file distributions")?;

    Ok(())
}
