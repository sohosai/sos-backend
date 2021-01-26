use crate::model::project::Project;

use anyhow::Result;
use futures::stream::{BoxStream, StreamExt, TryStreamExt};

pub fn list_projects_by_owner<'a, E>(conn: E, owner_id: String) -> BoxStream<'a, Result<Project>>
where
    E: sqlx::Executor<'a, Database = sqlx::Postgres> + 'a,
{
    sqlx::query_as("SELECT * FROM projects WHERE owner_id = $1")
        .bind(owner_id)
        .fetch(conn)
        .map_err(anyhow::Error::msg)
        .boxed()
}
