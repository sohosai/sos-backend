use crate::model::pending_project::PendingProject;

use anyhow::{Context, Result};
use uuid::Uuid;

pub async fn find_pending_project<'a, E>(conn: E, id: Uuid) -> Result<Option<PendingProject>>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres>,
{
    sqlx::query_as_unchecked!(
        PendingProject,
        "SELECT * FROM pending_projects WHERE id = $1",
        id
    )
    .fetch_optional(conn)
    .await
    .context("Failed to select from pending projects")
}
