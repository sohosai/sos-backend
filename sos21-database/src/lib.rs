use anyhow::{Context, Result};

pub mod command;
pub mod model;
pub mod query;

pub async fn migrate<'a, A>(conn: A) -> Result<()>
where
    A: sqlx::Acquire<'a, Database = sqlx::Postgres>,
{
    sqlx::migrate!("./migrations")
        .run(conn)
        .await
        .context("Failed to run migrations")
}
