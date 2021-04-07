use crate::model::pending_project::PendingProject;

use anyhow::{Context, Result};
use futures::stream::{BoxStream, StreamExt};

pub fn list_pending_projects_by_user<'a, 'b, E>(
    conn: E,
    user_id: String,
) -> BoxStream<'b, Result<PendingProject>>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres> + 'b,
    'a: 'b,
{
    sqlx::query_as_unchecked!(
        PendingProject,
        "SELECT * FROM pending_projects WHERE author_id = $1",
        user_id
    )
    .fetch(conn)
    .map(|pending_project| pending_project.context("Failed to select from pending projects"))
    .boxed()
}
