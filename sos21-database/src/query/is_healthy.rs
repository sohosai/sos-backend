use anyhow::{Context, Result};

pub async fn is_healthy<'a, E>(conn: E) -> Result<bool>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    let not_in_sync = sqlx::query_scalar!(
        r#"SELECT coalesce(bool_or(needs_sync), false) as "not_in_sync!" FROM forms"#
    )
    .fetch_one(conn)
    .await
    .context("Failed to select from forms")?;

    Ok(!not_in_sync)
}
