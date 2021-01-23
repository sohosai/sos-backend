use crate::model::user::User;

use anyhow::{Context, Result};

pub async fn find_user<'a, E>(conn: E, id: String) -> Result<Option<User>>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    sqlx::query_as("SELECT * FROM users WHERE id = $1")
        .bind(id)
        .fetch_optional(conn)
        .await
        .context("Failed to select from users")
}
