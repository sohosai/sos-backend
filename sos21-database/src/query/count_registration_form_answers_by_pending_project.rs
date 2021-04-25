use std::convert::TryInto;

use anyhow::{Context, Result};
use uuid::Uuid;

pub async fn count_registration_form_answers_by_pending_project<'a, E>(
    conn: E,
    pending_project_id: Uuid,
) -> Result<u64>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    let count = sqlx::query_scalar!(
        "SELECT count(*) FROM registration_form_answers WHERE pending_project_id = $1",
        pending_project_id
    )
    .fetch_one(conn)
    .await
    .context("Failed to select from registration form answers")?;

    let count = count.unwrap_or(0).try_into()?;
    Ok(count)
}
