use std::convert::TryInto;

use anyhow::{Context, Result};

// TODO: use lightweight way to count projects
// still, this is ok for now because number of projects are
// no more than 1000 as a result (although it is not a knowledge of this layer)
pub async fn count_projects<'a, E>(conn: E) -> Result<u64>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    let count = sqlx::query_scalar!(r#"SELECT count(*) as "count!" FROM projects"#)
        .fetch_one(conn)
        .await
        .context("Failed to count projects")?;
    let count = count.try_into()?;
    Ok(count)
}
