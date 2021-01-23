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
        last_name,
        email,
        role,
    } = user;

    sqlx::query("INSERT INTO users ( id, created_at, first_name, last_name, email, role ) VALUES ( $1, $2, $3, $4, $5, $6 )")
        .bind(id)
        .bind(created_at)
        .bind(first_name)
        .bind(last_name)
        .bind(email)
        .bind(role)
        .execute(conn)
        .await
        .context("Failed to select from users")?;

    Ok(())
}
