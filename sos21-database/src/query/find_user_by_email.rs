use crate::model::user::User;

use anyhow::{Context, Result};

pub async fn find_user_by_email<'a, E, S>(conn: E, email: S) -> Result<Option<User>>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
    S: AsRef<str>,
{
    sqlx::query_as_unchecked!(User, "SELECT * FROM users WHERE email = $1", email.as_ref())
        .fetch_optional(conn)
        .await
        .context("Failed to select from users")
}
