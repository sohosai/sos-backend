use anyhow::{Context, Result};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Input {
    pub id: Uuid,
    pub object_id: Uuid,
    pub blake3_digest: Vec<u8>,
    pub name: Option<String>,
    pub type_: String,
    pub size: i64,
}

pub async fn update_file<'a, E>(conn: E, input: Input) -> Result<()>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    sqlx::query!(
        r#"
UPDATE files
  SET
    object_id = $2,
    blake3_digest = $3,
    name = $4,
    type_ = $5,
    size = $6
  WHERE id = $1
"#,
        input.id,
        input.object_id,
        input.blake3_digest,
        input.name,
        input.type_,
        input.size,
    )
    .execute(conn)
    .await
    .context("Failed to update on files")?;

    Ok(())
}
