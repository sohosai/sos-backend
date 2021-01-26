use crate::model::user::User;

use anyhow::{Context, Result};

pub async fn insert_user<'a, E>(conn: E, user: User) -> Result<()>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    let User {
        id,
        created_at,
        first_name,
        kana_first_name,
        last_name,
        kana_last_name,
        email,
        role,
    } = user;

    sqlx::query!(
        r#"
INSERT INTO users (
    id,
    created_at,
    first_name,
    kana_first_name,
    last_name,
    kana_last_name,
    email,
    role
) VALUES ( $1, $2, $3, $4, $5, $6, $7, $8 )
"#,
        id,
        created_at,
        first_name,
        kana_first_name,
        last_name,
        kana_last_name,
        email,
        role as _
    )
    .execute(conn)
    .await
    .context("Failed to insert to users")?;

    Ok(())
}
