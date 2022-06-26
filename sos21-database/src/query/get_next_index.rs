use std::convert::TryInto;

use anyhow::{Context, Result};

pub async fn get_next_index<'a, E>(conn: E) -> Result<u64>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    let index = sqlx::query_scalar!(r#"SELECT max(index)+1 as "index" FROM projects"#)
        .fetch_one(conn)
        .await
        .context("Failed to get next index")?;

    let index = match index {
        Some(x) => x.try_into()?,
        None => 0,
    };

    Ok(index)
}
