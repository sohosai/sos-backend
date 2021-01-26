use crate::model::user::UserRole;

use anyhow::{Context, Result};

#[derive(Debug, Clone)]
pub struct Input {
    pub id: String,
    pub first_name: String,
    pub kana_first_name: String,
    pub last_name: String,
    pub kana_last_name: String,
    pub role: UserRole,
}

pub async fn update_user<'a, E>(conn: E, input: Input) -> Result<()>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    sqlx::query(
        r#"
UPDATE users
  SET
    first_name = $2,
    kana_first_name = $3,
    last_name = $4,
    kana_last_name = $5,
    role = $6
  WHERE id = $1
"#,
    )
    .bind(input.id)
    .bind(input.first_name)
    .bind(input.kana_first_name)
    .bind(input.last_name)
    .bind(input.kana_last_name)
    .bind(input.role)
    .execute(conn)
    .await
    .context("Failed to update on users")?;
    Ok(())
}
